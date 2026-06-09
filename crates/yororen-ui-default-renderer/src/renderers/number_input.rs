//! `NumberInputRenderer` — visual side of `NumberInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` (the inner
//! painter). The caller owns the canonical numeric value; the
//! renderer's on_change fires with the parsed `f64` (or the
//! current value if parsing fails). `-` / `+` stepper buttons
//! at the trailing edge call `on_decrement` / `on_increment`.

use std::any::Any;
use std::sync::Arc;

use gpui::{
    div, px, AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window,
};
use yororen_ui_core::headless::number_input::NumberInputProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::spec::Edges;
use crate::renderers::text_input::{
    start_cursor_blink, wire_input_keyboard, TextInputElement,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct NumberInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait NumberInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn stepper_button_size(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenNumberInputRenderer;

impl NumberInputRenderer for TokenNumberInputRenderer {
    fn bg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn focus_border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn hover_border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn min_height(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.number_input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme.get_number("tokens.control.number_input.horizontal_padding").unwrap_or(0.0) as f32),
            px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn stepper_button_size(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.number_input.stepper_button_size").unwrap_or(0.0) as f32)
    }
    fn border_radius(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_number_input<T: NumberInputRenderer + 'static>(r: T) -> Arc<dyn NumberInputRenderer> {
    Arc::new(r)
}

pub trait DefaultNumberInput: Sized {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement;
}

impl DefaultNumberInput for NumberInputProps {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement {
        let theme_arc = cx.theme().clone();
                let r: Arc<dyn NumberInputRenderer> = cx
            .renderer_arc::<markers::NumberInput, dyn NumberInputRenderer>()
            .expect("NumberInputRenderer registered").clone();
        let theme = &*theme_arc;

        let id = self.id.clone();
        let placeholder_str = self.placeholder.clone();
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let on_change_for_state = on_change.clone();
        let on_change_for_dec = on_change.clone();
        let on_change_for_inc = on_change.clone();
        let on_increment = self.on_increment.clone();
        let on_decrement = self.on_decrement.clone();
        let value = self.value;
        let step = self.step;
        let min = self.min;
        let max = self.max;

        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        // Initialise the state's value with the formatted numeric value
        // (one-time — user typing overrides this).
        if state.read(cx).value.is_empty() {
            state.update(cx, |s, _cx| {
                s.value = format!("{}", value);
                s.caret = s.value.len();
            });
        }
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(placeholder_str);
            s.on_change = Some(Arc::new(move |new_value: &str, window: &mut gpui::Window, cx: &mut gpui::App| {
                if let Some(cb) = on_change_for_state.as_ref() {
                    let parsed = new_value.parse::<f64>().unwrap_or(value);
                    cb(parsed, window, cx);
                }
            }));
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);

        let render_state = NumberInputRenderState {
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
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let stepper_size = r.stepper_button_size(&render_state, theme);

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
            None, // no on_submit for numbers
        );

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);

        let on_inc_clone = on_increment.clone();
        let on_dec_clone = on_decrement.clone();
        // The steppers mutate the live `state.value` first, then
        // fire `on_change` with the new value, then call
        // `on_increment` / `on_decrement` for the caller to do
        // extra work (e.g. update an external state). Without
        // the live `state.value` write, the input would still
        // display the old value after the click.
        let state_for_dec = state.clone();
        let state_for_inc = state.clone();
        let final_div = keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .child(
                div()
                    .size(stepper_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("−")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        let next = value - step;
                        let clamped = match min {
                            Some(m) => next.max(m),
                            None => next,
                        };
                        let new_text = format!("{}", clamped);
                        state_for_dec.update(cx, |s, cx| {
                            s.value = new_text.clone();
                            s.caret = new_text.len();
                            s.selection_start = new_text.len();
                            s.selection_end = new_text.len();
                            cx.notify();
                        });
                        if let Some(cb) = on_change_for_dec.as_ref() {
                            cb(clamped, window, cx);
                        }
                        if let Some(cb) = on_dec_clone.as_ref() {
                            cb(clamped, window, cx);
                        }
                    }),
            )
            .child(
                div()
                    .size(stepper_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("+")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        let next = value + step;
                        let clamped = match max {
                            Some(m) => next.min(m),
                            None => next,
                        };
                        let new_text = format!("{}", clamped);
                        state_for_inc.update(cx, |s, cx| {
                            s.value = new_text.clone();
                            s.caret = new_text.len();
                            s.selection_start = new_text.len();
                            s.selection_end = new_text.len();
                            cx.notify();
                        });
                        if let Some(cb) = on_change_for_inc.as_ref() {
                            cb(clamped, window, cx);
                        }
                        if let Some(cb) = on_inc_clone.as_ref() {
                            cb(clamped, window, cx);
                        }
                    }),
            );

        final_div.into_any_element()
    }
}
