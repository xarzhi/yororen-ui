//! `PasswordInputRenderer` — visual side of `PasswordInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` (the inner
//! painter) but the renderer shows the `mask_char` repeated for
//! the value's char count instead of the raw value. The real
//! value lives in `TextInputState.value`; only the *display* is
//! masked.

use std::any::Any;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use yororen_ui_core::headless::password_input::PasswordInputProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::text_input::{TextInputElement, start_cursor_blink, wire_input_keyboard};
pub use yororen_ui_core::renderer::password_input::{
    PasswordInputRenderState, PasswordInputRenderer,
};
use yororen_ui_core::renderer::spec::Edges;

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
    fn hover_border(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
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
        px(theme
            .get_number("tokens.control.input.min_height")
            .unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_password_input<T: PasswordInputRenderer + 'static>(
    r: T,
) -> Arc<dyn PasswordInputRenderer> {
    Arc::new(r)
}
