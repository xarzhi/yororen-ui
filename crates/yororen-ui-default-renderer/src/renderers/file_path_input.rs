//! `FilePathInputRenderer` — visual side of `FilePathInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` plus a folder
//! icon at the leading edge and a "browse" button at the
//! trailing edge. The browse button calls the caller's
//! `on_browse` callback (which is expected to open a file
//! dialog via the platform API).

use std::any::Any;
use std::sync::Arc;

use gpui::{
    div, px, AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window,
};
use yororen_ui_core::headless::file_path_input::FilePathInputProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::{ActiveTheme, Theme};

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
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    fn button_fg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
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
            hint_color: text_color,
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
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
        let final_div = keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(div().size(icon_size).flex().items_center().justify_center().child("📁"))
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .child(
                div()
                    .size(icon_size)
                    .bg(button_bg)
                    .rounded(px(4.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(button_fg)
                    .child("…")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        if let Some(cb) = on_browse_clone.as_ref() {
                            cb(window, cx);
                        }
                    }),
            );

        final_div.into_any_element()
    }
}
