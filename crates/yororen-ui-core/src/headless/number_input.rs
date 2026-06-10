//! Headless `number_input` — numeric input with stepper buttons.
//!
//! Reuses `TextInputState` (renderer-minted); the renderer
//! parses the text to `f64` and adds +/- stepper buttons.

use std::sync::Arc;

use gpui::{App, Hsla};

pub type NumberChangeCallback = Arc<dyn Fn(f64, &mut gpui::Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct NumberInputProps {
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: f64,
    pub value: f64,
    pub on_change: Option<NumberChangeCallback>,
    pub on_increment: Option<NumberChangeCallback>,
    pub on_decrement: Option<NumberChangeCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn number_input(id: impl Into<gpui::ElementId>) -> NumberInputProps {
    NumberInputProps {
        id: id.into(),
        placeholder: String::new(),
        disabled: false,
        min: None,
        max: None,
        step: 1.0,
        value: 0.0,
        on_change: None,
        on_increment: None,
        on_decrement: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl NumberInputProps {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn min(mut self, v: f64) -> Self {
        self.min = Some(v);
        self
    }
    pub fn max(mut self, v: f64) -> Self {
        self.max = Some(v);
        self
    }
    pub fn step(mut self, v: f64) -> Self {
        self.step = v;
        self
    }
    pub fn value(mut self, v: f64) -> Self {
        self.value = v;
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f64, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_increment<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f64, &mut gpui::Window, &mut App),
    {
        self.on_increment = Some(Arc::new(f));
        self
    }
    pub fn on_decrement<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f64, &mut gpui::Window, &mut App),
    {
        self.on_decrement = Some(Arc::new(f));
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn has_custom_border(mut self, v: bool) -> Self {
        self.has_custom_border = v;
        self
    }
    pub fn has_custom_focus_border(mut self, v: bool) -> Self {
        self.has_custom_focus_border = v;
        self
    }
    pub fn custom_bg(mut self, c: Hsla) -> Self {
        self.custom_bg = Some(c);
        self.has_custom_bg = true;
        self
    }
    pub fn custom_border(mut self, c: Hsla) -> Self {
        self.custom_border = Some(c);
        self.has_custom_border = true;
        self
    }
    pub fn custom_focus_border(mut self, c: Hsla) -> Self {
        self.custom_focus_border = Some(c);
        self.has_custom_focus_border = true;
        self
    }
    pub fn custom_text_color(mut self, c: Hsla) -> Self {
        self.custom_text_color = Some(c);
        self
    }

    /// Render the number input using the registered `NumberInputRenderer`.
    pub fn render(self, cx: &mut gpui::App, window: &mut gpui::Window) -> gpui::AnyElement {
        use crate::headless::text_input::TextInputState;
        use crate::headless::text_input_element::{
            TextInputElement, start_cursor_blink, wire_input_keyboard,
        };
        use crate::renderer::RendererContext;
        use crate::renderer::markers::NumberInput as NumberInputMarker;
        use crate::renderer::number_input::{NumberInputRenderState, NumberInputRenderer};
        use crate::renderer::spec::Edges;
        use crate::theme::ActiveTheme;
        use gpui::{
            CursorStyle, InteractiveElement, IntoElement, MouseButton, ParentElement, Stateful,
            StatefulInteractiveElement, Styled, div, px,
        };
        use std::sync::Arc;

        let theme_arc = cx.theme().clone();
        let r: Arc<dyn NumberInputRenderer> = cx
            .renderer_arc::<NumberInputMarker, dyn NumberInputRenderer>()
            .expect("NumberInputRenderer registered")
            .clone();
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
        if state.read(cx).value.is_empty() {
            state.update(cx, |s, _cx| {
                s.value = format!("{}", value);
                s.caret = s.value.len();
            });
        }
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(placeholder_str);
            s.on_change = Some(Arc::new(
                move |new_value: &str, window: &mut gpui::Window, cx: &mut gpui::App| {
                    if let Some(cb) = on_change_for_state.as_ref() {
                        let parsed = new_value.parse::<f64>().unwrap_or(value);
                        cb(parsed, window, cx);
                    }
                },
            ));
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
        let padding: Edges<gpui::Pixels> = r.padding(&render_state, theme);
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

        let base: Stateful<gpui::Div> = div()
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
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            });

        let focused_div: Stateful<gpui::Div> = base.track_focus(&focus_handle);
        let keyed = wire_input_keyboard(
            focused_div,
            state.clone(),
            focus_handle.clone(),
            disabled,
            None,
        );

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);

        let on_inc_clone = on_increment.clone();
        let on_dec_clone = on_decrement.clone();
        let state_for_dec = state.clone();
        let state_for_inc = state.clone();

        keyed
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
            )
            .into_any_element()
    }
}
