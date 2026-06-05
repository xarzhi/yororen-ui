//! `TextInputRenderer` — visual side of `TextInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    /// Caller-supplied overrides. When the corresponding
    /// `has_custom_*` is true, the renderer returns this color
    /// instead of the built-in token path.
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn hint_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TextInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TextInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &TextInputRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &TextInputRenderState, theme: &Theme) -> f32;
}

pub struct TokenTextInputRenderer;

impl TextInputRenderer for TokenTextInputRenderer {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
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
    fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
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
    fn focus_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_focus_border {
            state
                .custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        }
    }
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.custom_text_color.is_some() {
            state.custom_text_color.unwrap()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    fn hint_color(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    fn min_height(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &TextInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.control.input.horizontal_padding").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn disabled_opacity(&self, state: &TextInputRenderState, _theme: &Theme) -> f32 {
        if state.disabled { 0.6 } else { 1.0 }
    }
}

pub fn arc_text_input<T: TextInputRenderer + 'static>(r: T) -> Arc<dyn TextInputRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultTextInput` — `headless::TextInputProps` sugar.
// =====================================================================

use gpui::{
    div, App, InteractiveElement, KeyDownEvent, MouseButton, ParentElement, Stateful, Styled,
    Window,
};
use yororen_ui_core::headless::text_input::TextInputProps;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultTextInput: Sized {
    /// `default_render` takes both `&App` and `&Window` so it
    /// can query `FocusHandle::is_focused(&window)` and pick
    /// between the focused and unfocused border color. The
    /// trait deliberately takes both — `text_input` is the
    /// only `DefaultXxx` that needs window access, and the
    /// caller has both available in a `Render::render` closure
    /// anyway.
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div>;
}

impl DefaultTextInput for TextInputProps {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn TextInputRenderer> = cx
            .renderer_arc::<markers::TextInput, dyn TextInputRenderer>()
            .expect("TextInputRenderer registered");

        // Read state up front so closures can capture the entity
        // by move (entity cloning is cheap; it's an Arc).
        let state = self.state.clone();
        let focus_handle = self.focus_handle.clone();
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let placeholder = self.placeholder.clone();
        let max_length = self.max_length;
        let disabled = self.disabled;

        // We have `&Window` now (the trait took it for this
        // reason) — query the live focus state so the border
        // colour swaps to `focus_border` when the user clicks
        // into the input.
        let focused = focus_handle.is_focused(window);

        let render_state = TextInputRenderState {
            disabled,
            focused,
            has_custom_bg: self.has_custom_bg,
            has_custom_border: self.has_custom_border,
            has_custom_focus_border: self.has_custom_focus_border,
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
        let hint_color = r.hint_color(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let opacity = r.disabled_opacity(&render_state, theme);

        // Snapshot the current value for the child render.
        // The caller is expected to call `cx.notify()` from
        // their `on_change` callback so gpui re-renders with
        // the new value; this is the standard reactive pattern.
        let value = state.read(cx).value.clone();

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

        // Visible text — placeholder when value is empty,
        // value otherwise. The hint color is applied separately
        // so we don't need to retint the parent.
        if value.is_empty() {
            el = el.child(div().flex_1().text_color(hint_color).child(placeholder));
        } else {
            el = el.child(div().flex_1().child(value));
        }

        // Mouse-down focuses the input.
        let focus_for_mouse = focus_handle.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_ev, window, _cx| {
            focus_for_mouse.focus(window);
        });

        // Key dispatch — only when focused and not disabled.
        if !disabled {
            let state_for_keystroke = state.clone();
            let on_change_for_keystroke = on_change.clone();
            let on_submit_for_keystroke = on_submit.clone();
            el = el.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let keystroke = &ev.keystroke;
                if keystroke.key.as_str() == "enter" {
                    if let Some(cb) = on_submit_for_keystroke.as_ref() {
                        let snapshot = state_for_keystroke.read(cx).value.clone();
                        cb(&snapshot, window, cx);
                    }
                    return;
                }
                match keystroke.key.as_str() {
                    "backspace" => {
                        let new_value = state_for_keystroke
                            .update(cx, |s, _cx| {
                                s.backspace();
                                s.value.clone()
                            });
                        if let Some(cb) = on_change_for_keystroke.as_ref() {
                            cb(&new_value, window, cx);
                        }
                    }
                    "delete" => {
                        let new_value = state_for_keystroke
                            .update(cx, |s, _cx| {
                                s.delete_forward();
                                s.value.clone()
                            });
                        if let Some(cb) = on_change_for_keystroke.as_ref() {
                            cb(&new_value, window, cx);
                        }
                    }
                    "left" => {
                        state_for_keystroke.update(cx, |s, _cx| s.move_caret_left());
                    }
                    "right" => {
                        state_for_keystroke.update(cx, |s, _cx| s.move_caret_right());
                    }
                    "home" => {
                        state_for_keystroke.update(cx, |s, _cx| s.move_caret_to_start());
                    }
                    "end" => {
                        state_for_keystroke.update(cx, |s, _cx| s.move_caret_to_end());
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
                                let cur = state_for_keystroke.read(cx).value.len();
                                if cur + to_insert.len() > cap {
                                    return;
                                }
                            }
                            let new_value = state_for_keystroke.update(cx, |s, _cx| {
                                s.insert_text(&to_insert);
                                s.value.clone()
                            });
                            if let Some(cb) = on_change_for_keystroke.as_ref() {
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
