//! `FilePathInputRenderer` — visual side of `FilePathInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` plus a folder
//! icon at the leading edge and a "browse" button at the
//! trailing edge. Clicking the browse button opens a native
//! file dialog via `App::prompt_for_paths`; the chosen path
//! is written to the state and fired through `on_change`.
//! The user's `on_browse` becomes a post-pick hook that
//! receives the selected path (empty string on cancel).

use std::any::Any;
use std::sync::Arc;

use gpui::{
    div, px, AnyElement, App, AppContext, Div, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window,
};
use yororen_ui_core::headless::file_path_input::FilePathInputProps;
use yororen_ui_core::headless::icon::{icon, IconSource};
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::icon::DefaultIcon;
use crate::renderers::spec::Edges;
use crate::renderers::text_input::{
    start_cursor_blink, wire_input_keyboard, TextInputElement,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct FilePathInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait FilePathInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn button_bg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn button_hover_bg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn button_fg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &FilePathInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &FilePathInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn action_gap(&self, state: &FilePathInputRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &FilePathInputRenderState, theme: &Theme) -> Pixels;
    fn icon_size(&self, state: &FilePathInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenFilePathInputRenderer;

impl FilePathInputRenderer for TokenFilePathInputRenderer {
    fn bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn focus_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn hover_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn button_bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        // The browse button is a small affordance inside the
        // input, not a primary action. Match the input surface
        // (white in light, dark gray in dark) so the icon
        // doesn't compete with the typed path.
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn button_fg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        // Use the same color as the typed text. On hover, the
        // theme can override via `custom_button_fg` or by
        // registering a custom `FilePathInputRenderer`.
        theme.get_color("content.primary").unwrap_or_default()
    }
    fn button_hover_bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    fn min_height(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.file_path_input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme.get_number("tokens.control.file_path_input.horizontal_padding").unwrap_or(0.0) as f32),
            px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn action_gap(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.file_path_input.action_gap").unwrap_or(0.0) as f32)
    }
    fn border_radius(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn icon_size(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.file_path_input.icon_size").unwrap_or(0.0) as f32)
    }
}

pub fn arc_file_path_input<T: FilePathInputRenderer + 'static>(r: T) -> Arc<dyn FilePathInputRenderer> {
    Arc::new(r)
}

pub trait DefaultFilePathInput: Sized {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement;
}

impl DefaultFilePathInput for FilePathInputProps {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement {
        // Copy the theme Arc up front so the `cx.theme()` borrow
        // doesn't conflict with later `cx.renderer_arc` /
        // `cx.use_keyed_state` mutable calls.
        let theme_arc = cx.theme().clone();
        let r: Arc<dyn FilePathInputRenderer> = cx
            .renderer_arc::<markers::FilePathInput, dyn FilePathInputRenderer>()
            .expect("FilePathInputRenderer registered").clone();
        let theme = &*theme_arc;

        let id = self.id.clone();
        let placeholder_str = self.placeholder.clone();
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let on_browse = self.on_browse.clone();

        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(placeholder_str);
            s.on_change = on_change.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);

        let render_state = FilePathInputRenderState {
            disabled,
            focused,
            custom_bg: self.custom_bg,
            custom_border: self.custom_border,
            custom_focus_border: self.custom_focus_border,
            custom_fg: self.custom_text_color,
        };
        let bg = r.bg(&render_state, theme);
        let border_color = if focused {
            r.focus_border(&render_state, theme)
        } else {
            r.border(&render_state, theme)
        };
        let text_color = theme.get_color("content.primary").unwrap_or_default();
        let button_fg = r.button_fg(&render_state, theme);
        let button_bg = r.button_bg(&render_state, theme);
        let button_hover_bg = r.button_hover_bg(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let action_gap = r.action_gap(&render_state, theme);
        let icon_size = r.icon_size(&render_state, theme);

        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color: theme.get_color("content.tertiary").unwrap_or_default(),
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
            value_override: None,
        };

        let base: Stateful<Div> = div()
            .id(id.clone())
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .gap(action_gap)
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                gpui::CursorStyle::Arrow
            } else {
                gpui::CursorStyle::IBeam
            });

        let focused_div: Stateful<Div> = base.track_focus(&focus_handle);
        let keyed = wire_input_keyboard(
            focused_div,
            state.clone(),
            focus_handle.clone(),
            disabled,
            None,
        );

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);

        let on_browse_clone = on_browse.clone();
        let state_for_browse = state.clone();
        let window_handle = window.window_handle();
        let final_div = keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(
                icon(
                    "file-path-input-leading-icon",
                    IconSource::Builtin("folder".into()),
                    &mut *cx,
                )
                .size(icon_size)
                .color(text_color)
                .default_render(&mut *cx, window),
            )
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .child(
                div()
                    .id("file-path-input-browse")
                    .size(icon_size)
                    .bg(button_bg)
                    .rounded(px(4.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(button_fg)
                    .hover(|s| s.bg(button_hover_bg))
                    .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
                        if disabled {
                            return;
                        }
                        // Open the native file dialog and let
                        // the user pick a single file. The
                        // result is delivered asynchronously
                        // via a oneshot channel, so we spawn
                        // a task to update the state when it
                        // arrives.
                        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
                            files: true,
                            directories: false,
                            multiple: false,
                            prompt: Some("Select a file".into()),
                        });
                        // Pull `on_change` out of the state
                        // *now*, while the outer `cx` is still
                        // in scope, then move the cloned value
                        // into the spawn future (which must be
                        // 'static and can't borrow the outer
                        // `cx`).
                        let on_change_for_async = state.read(cx).on_change.clone();
                        let state_for_browse = state.clone();
                        let on_browse_cb = on_browse_clone.clone();
                        cx.spawn(async move |async_cx| {
                            // `receiver.await` returns
                            // `Result<Result<Option<Vec<PathBuf>>>, _>`
                            // because the platform layer wraps its
                            // own Result inside the oneshot's Result.
                            let picked = match receiver.await {
                                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                                _ => None,
                            };
                            if let Some(path) = picked {
                                let path_str = path.to_string_lossy().to_string();
                                let state_for_change = state_for_browse.clone();
                                let on_browse_for_async = on_browse_cb.clone();
                                let _ = async_cx
                                    .update_window(window_handle, move |_, window, cx| {
                                        state_for_change.update(cx, |s, cx| {
                                            s.value = path_str.clone();
                                            let end = s.value.len();
                                            s.caret = end;
                                            s.selection_start = end;
                                            s.selection_end = end;
                                            cx.notify();
                                        });
                                        // The state's on_change was set
                                        // up by `default_render` from the
                                        // props; firing it here keeps the
                                        // browse-pick path in sync with the
                                        // user-typed path so the caller's
                                        // listener fires either way.
                                        if let Some(cb) = on_change_for_async.as_ref() {
                                            cb(&path_str, window, cx);
                                        }
                                        if let Some(cb) = on_browse_for_async.as_ref() {
                                            cb(&path_str, window, cx);
                                        }
                                    });
                            }
                        })
                        .detach();
                    })
                    .child(
                        icon(
                            "file-path-input-browse-icon",
                            IconSource::Builtin("file".into()),
                            &mut *cx,
                        )
                        .size(icon_size)
                        .color(button_fg)
                        .default_render(&mut *cx, window),
                    ),
            );

        final_div.into_any_element()
    }
}
