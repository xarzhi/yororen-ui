//! `TextAreaRenderer` — visual side of `TextArea`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextAreaRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextAreaRenderer: Any + Send + Sync {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TextAreaRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &TextAreaRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenTextAreaRenderer;

impl TextAreaRenderer for TokenTextAreaRenderer {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    fn focus_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else {
            state
                .custom_text_color
                .unwrap_or_else(|| theme.get_color("content.primary").unwrap_or_default())
        }
    }
    fn min_height(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.input.text_area_min_h").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32,
        ))
    }
    fn border_radius(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_text_area<T: TextAreaRenderer + 'static>(r: T) -> Arc<dyn TextAreaRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultTextArea` — `headless::TextAreaProps` sugar.
// =====================================================================

use gpui::{
    div, App, InteractiveElement, KeyDownEvent, MouseButton, ParentElement, Stateful, Styled,
    Window,
};
use yororen_ui_core::headless::text_area::TextAreaProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultTextArea: Sized {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div>;
}

impl DefaultTextArea for TextAreaProps {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn TextAreaRenderer> = cx
            .renderer_arc::<markers::TextArea, dyn TextAreaRenderer>()
            .expect("TextAreaRenderer registered");

        let state = self.state.clone();
        let focus_handle = self.focus_handle.clone();
        let on_change = self.on_change.clone();
        let placeholder = self.placeholder.clone();
        let max_length = self.max_length;
        let disabled = self.disabled;
        let focused = focus_handle.is_focused(window);

        let render_state = TextAreaRenderState {
            disabled,
            focused,
            has_custom_bg: self.has_custom_bg,
            custom_bg: self.custom_bg,
            custom_border: self.custom_border,
            custom_focus_border: self.custom_focus_border,
            custom_text_color: self.custom_text_color,
        };
        let bg = r.bg(&render_state, theme);
        let border_color = if focused {
            r.focus_border(&render_state, theme)
        } else {
            r.border(&render_state, theme)
        };
        let text_color = r.text_color(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);

        let value = state.read(cx).value.clone();
        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .p(padding.top)
            .flex()
            .items_start()
            .text_color(text_color);

        if value.is_empty() {
            el = el.child(div().flex_1().text_color(r.text_color(&render_state, theme)).child(placeholder));
        } else {
            el = el.child(div().flex_1().child(value));
        }

        // Mouse-down focuses the input.
        let focus_for_mouse = focus_handle.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_ev, window, _cx| {
            focus_for_mouse.focus(window);
        });

        // Key dispatch. Enter inserts a newline (multi-line
        // behaviour); no on_submit for text areas.
        if !disabled {
            let state_for_keys = state.clone();
            let on_change_for_keys = on_change.clone();
            el = el.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let keystroke = &ev.keystroke;
                match keystroke.key.as_str() {
                    "enter" => {
                        // Multi-line: insert newline.
                        if let Some(cap) = max_length {
                            let cur = state_for_keys.read(cx).value.len();
                            if cur + 1 > cap {
                                return;
                            }
                        }
                        let new_value = state_for_keys.update(cx, |s, _cx| {
                            s.insert_text("\n");
                            s.value.clone()
                        });
                        if let Some(cb) = on_change_for_keys.as_ref() {
                            cb(&new_value, window, cx);
                        }
                    }
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
