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
    AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};
use yororen_ui_core::headless::number_input::NumberInputProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::text_input::{TextInputElement, start_cursor_blink, wire_input_keyboard};
pub use yororen_ui_core::renderer::number_input::{NumberInputRenderState, NumberInputRenderer};
use yororen_ui_core::renderer::spec::Edges;

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
        px(theme
            .get_number("tokens.control.number_input.min_height")
            .unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.number_input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    fn stepper_button_size(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.number_input.stepper_button_size")
            .unwrap_or(0.0) as f32)
    }
    fn border_radius(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_number_input<T: NumberInputRenderer + 'static>(r: T) -> Arc<dyn NumberInputRenderer> {
    Arc::new(r)
}
