//! `NumberInputRenderer` — visual side of `NumberInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

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
    fn stepper_bg(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn stepper_fg(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn stepper_button_size(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn stepper_icon_size(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
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
    fn stepper_bg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    fn stepper_fg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    fn min_height(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.number_input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.control.number_input.horizontal_padding").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn stepper_button_size(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.number_input.stepper_button_size").unwrap_or(0.0) as f32)
    }
    fn stepper_icon_size(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.number_input.stepper_icon_size").unwrap_or(0.0) as f32)
    }
    fn border_radius(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_number_input<T: NumberInputRenderer + 'static>(r: T) -> Arc<dyn NumberInputRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultNumberInput` — `headless::NumberInputProps` sugar.
// =====================================================================

use gpui::{
    div, App, InteractiveElement, MouseButton, ParentElement, Stateful, Styled, Window,
};
use yororen_ui_core::headless::number_input::NumberInputProps;
use yororen_ui_core::renderer::RendererContext;
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultNumberInput: Sized {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div>;
}

impl DefaultNumberInput for NumberInputProps {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn NumberInputRenderer> = cx
            .renderer_arc::<yororen_ui_core::renderer::markers::NumberInput, dyn NumberInputRenderer>(
            )
            .expect("NumberInputRenderer registered");

        let state = NumberInputRenderState {
            disabled: self.disabled,
            focused: self.focus_handle.is_focused(window),
            custom_bg: self.custom_bg,
            custom_border: self.custom_border,
            custom_focus_border: self.custom_focus_border,
            custom_fg: self.custom_text_color,
        };
        let bg = r.bg(&state, theme);
        let border_color = r.border(&state, theme);
        let min_h = r.min_height(&state, theme);
        let padding = r.padding(&state, theme);
        let stepper_size = r.stepper_button_size(&state, theme);
        let radius = r.border_radius(&state, theme);

        // Display the current numeric value as text. We re-read
        // the underlying text-input state to honour user edits.
        let value_text = state_to_value_text(&self, cx);

        let focus_for_mouse = self.focus_handle.clone();
        let on_inc = self.on_increment.clone();
        let on_dec = self.on_decrement.clone();
        let on_change = self.on_change.clone();
        let step = self.step;
        let min = self.min;
        let max = self.max;
        let value = self.value;

        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .px(padding.left)
            .flex()
            .items_center()
            .text_color(r.border(&state, theme))
            .child(div().flex_1().child(value_text));

        if !self.disabled {
            // Decrement stepper.
            let on_dec_clone = on_dec.clone();
            let step_dec = step;
            let min_dec = min;
            let value_dec = value;
            el = el.child(
                div()
                    .size(stepper_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("-")
                    .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
                        let next = value_dec - step_dec;
                        let clamped = match min_dec {
                            Some(m) => next.max(m),
                            None => next,
                        };
                        if let Some(cb) = on_dec_clone.as_ref() {
                            cb(clamped, _window, cx);
                        }
                    }),
            );
            // Increment stepper.
            let on_inc_clone = on_inc.clone();
            let step_inc = step;
            let max_inc = max;
            let value_inc = value;
            el = el.child(
                div()
                    .size(stepper_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("+")
                    .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
                        let next = value_inc + step_inc;
                        let clamped = match max_inc {
                            Some(m) => next.min(m),
                            None => next,
                        };
                        if let Some(cb) = on_inc_clone.as_ref() {
                            cb(clamped, _window, cx);
                        }
                    }),
            );
        }

        // Mouse-down on the whole input focuses it.
        el = el.on_mouse_down(MouseButton::Left, move |_ev, window, _cx| {
            focus_for_mouse.focus(window);
        });

        // Note: we don't wire text editing through the
        // NumberInputState — the caller is expected to track
        // the numeric value in their own state entity and
        // call `value(v)` on next render. `on_change` fires
        // when the user finishes editing (focus loss / Enter
        // / stepper press).
        let _ = on_change;
        let _ = state;
        self.apply(el)
    }
}

fn state_to_value_text(props: &NumberInputProps, cx: &App) -> String {
    // Prefer the underlying text-input state if non-empty
    // (so the user can type negative numbers etc.); fall
    // back to the numeric value formatted as a string.
    let text_state = props.state.read(cx);
    if !text_state.value.is_empty() {
        text_state.value.clone()
    } else {
        format!("{}", props.value)
    }
}
