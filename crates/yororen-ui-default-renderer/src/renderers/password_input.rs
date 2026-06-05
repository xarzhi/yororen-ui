//! `PasswordInputRenderer` — visual side of `PasswordInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct PasswordInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait PasswordInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
    fn toggle_icon_size(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenPasswordInputRenderer;

impl PasswordInputRenderer for TokenPasswordInputRenderer {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else if state.has_custom_bg {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else if state.has_custom_border {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        } else {
            theme.get_color("border.default").unwrap_or_default()
        }
    }
    fn focus_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_focus_border {
            state
                .custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        }
    }
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else {
            state
                .custom_fg
                .unwrap_or_else(|| theme.get_color("content.primary").unwrap_or_default())
        }
    }
    fn min_height(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.control.input.horizontal_padding").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn toggle_icon_size(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.sizes.icon_sm").unwrap_or(0.0) as f32)
    }
}

pub fn arc_password_input<T: PasswordInputRenderer + 'static>(
    r: T,
) -> Arc<dyn PasswordInputRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultPasswordInput` — `headless::PasswordInputProps` sugar.
// =====================================================================

use gpui::{
    div, App, InteractiveElement, KeyDownEvent, MouseButton, ParentElement, Stateful, Styled,
    Window,
};
use yororen_ui_core::headless::password_input::PasswordInputProps;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultPasswordInput: Sized {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div>;
}

impl DefaultPasswordInput for PasswordInputProps {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn PasswordInputRenderer> = cx
            .renderer_arc::<markers::PasswordInput, dyn PasswordInputRenderer>()
            .expect("PasswordInputRenderer registered");

        let state = self.state.clone();
        let focus_handle = self.focus_handle.clone();
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let placeholder = self.placeholder.clone();
        let max_length = self.max_length;
        let disabled = self.disabled;
        let mask_char = self.mask_char;
        let focused = focus_handle.is_focused(window);

        let render_state = PasswordInputRenderState {
            disabled,
            focused,
            has_custom_bg: self.has_custom_bg,
            has_custom_border: self.has_custom_border,
            has_custom_focus_border: self.has_custom_focus_border,
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
        let text_color = r.fg(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let opacity = 1.0;

        // Build the masked display string (e.g. "•••••" for
        // 5 chars of value). The real value lives in the
        // TextInputState; the renderer shows the mask.
        let value_len = state.read(cx).value.chars().count();
        let masked: String = std::iter::repeat(mask_char).take(value_len).collect();

        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .opacity(opacity)
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .text_color(text_color);

        if masked.is_empty() {
            el = el.child(div().flex_1().child(placeholder));
        } else {
            el = el.child(div().flex_1().child(masked));
        }

        let focus_for_mouse = focus_handle.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_ev, window, _cx| {
            focus_for_mouse.focus(window);
        });

        if !disabled {
            let state_for_keys = state.clone();
            let on_change_for_keys = on_change.clone();
            let on_submit_for_keys = on_submit.clone();
            el = el.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let keystroke = &ev.keystroke;
                if keystroke.key.as_str() == "enter" {
                    if let Some(cb) = on_submit_for_keys.as_ref() {
                        let snapshot = state_for_keys.read(cx).value.clone();
                        cb(&snapshot, window, cx);
                    }
                    return;
                }
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
                            if let Some(cap) = max_length {
                                let cur = state_for_keys.read(cx).value.len();
                                if cur + to_insert.len() > cap {
                                    return;
                                }
                            }
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
