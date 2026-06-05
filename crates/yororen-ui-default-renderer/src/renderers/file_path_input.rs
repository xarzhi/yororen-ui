//! `FilePathInputRenderer` — visual side of `FilePathInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

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
    fn button_bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    fn button_fg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    fn min_height(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.file_path_input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.control.file_path_input.horizontal_padding").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn action_gap(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.file_path_input.action_gap").unwrap_or(0.0) as f32)
    }
    fn border_radius(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn icon_size(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.file_path_input.icon_size").unwrap_or(0.0) as f32)
    }
}

pub fn arc_file_path_input<T: FilePathInputRenderer + 'static>(
    r: T,
) -> Arc<dyn FilePathInputRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultFilePathInput` — `headless::FilePathInputProps` sugar.
// =====================================================================

use gpui::{
    div, App, InteractiveElement, KeyDownEvent, MouseButton, ParentElement, Stateful, Styled,
    Window,
};
use yororen_ui_core::headless::file_path_input::FilePathInputProps;
use yororen_ui_core::renderer::RendererContext;
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultFilePathInput: Sized {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div>;
}

impl DefaultFilePathInput for FilePathInputProps {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn FilePathInputRenderer> = cx
            .renderer_arc::<yororen_ui_core::renderer::markers::FilePathInput, dyn FilePathInputRenderer>(
            )
            .expect("FilePathInputRenderer registered");

        let state = self.state.clone();
        let focus_handle = self.focus_handle.clone();
        let on_change = self.on_change.clone();
        let on_browse = self.on_browse.clone();
        let placeholder = self.placeholder.clone();
        let disabled = self.disabled;
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
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let action_gap = r.action_gap(&render_state, theme);
        let icon_size = r.icon_size(&render_state, theme);
        let button_size = icon_size; // share

        let value = state.read(cx).value.clone();
        let mut el = div()
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
            .text_color(r.button_fg(&render_state, theme));

        el = el.child(div().size(icon_size).flex().items_center().justify_center().child("📁"));

        if value.is_empty() {
            el = el.child(div().flex_1().text_color(r.button_fg(&render_state, theme)).child(placeholder));
        } else {
            el = el.child(div().flex_1().child(value));
        }

        // Browse button.
        let on_browse_clone = on_browse.clone();
        el = el.child(
            div()
                .size(button_size)
                .bg(r.button_bg(&render_state, theme))
                .rounded(px(4.0))
                .flex()
                .items_center()
                .justify_center()
                .child("…")
                .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                    if let Some(cb) = on_browse_clone.as_ref() {
                        cb(window, cx);
                    }
                }),
        );

        let focus_for_mouse = focus_handle.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_ev, window, _cx| {
            focus_for_mouse.focus(window);
        });

        if !disabled {
            let state_for_keys = state.clone();
            let on_change_for_keys = on_change.clone();
            el = el.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let keystroke = &ev.keystroke;
                match keystroke.key.as_str() {
                    "backspace" => {
                        let new_value = state_for_keys.update(cx, |s, _cx| {
                            s.backspace();
                            s.value.clone()
                        });
                        if let Some(cb) = on_change_for_keys.as_ref() {
                            cb(&new_value, window, cx);
                        }
                    }
                    "delete" => {
                        let new_value = state_for_keys.update(cx, |s, _cx| {
                            s.delete_forward();
                            s.value.clone()
                        });
                        if let Some(cb) = on_change_for_keys.as_ref() {
                            cb(&new_value, window, cx);
                        }
                    }
                    _ => {
                        let ch_opt: Option<&str> = keystroke
                            .key_char
                            .as_deref()
                            .filter(|s| !s.is_empty())
                            .or_else(|| {
                                if keystroke.key.is_empty() {
                                    None
                                } else {
                                    Some(keystroke.key.as_str())
                                }
                            });
                        let Some(ch) = ch_opt else { return };
                        if ch.chars().count() == 1
                            && !keystroke.modifiers.control
                            && !keystroke.modifiers.alt
                            && !keystroke.modifiers.platform
                        {
                            let to_insert = ch.to_string();
                            let new_value = state_for_keys.update(cx, |s, _cx| {
                                s.insert_text(&to_insert);
                                s.value.clone()
                            });
                            if let Some(cb) = on_change_for_keys.as_ref() {
                                cb(&new_value, window, cx);
                            }
                        }
                    }
                }
            });
        }

        self.apply(el)
    }
}

use gpui::px;
