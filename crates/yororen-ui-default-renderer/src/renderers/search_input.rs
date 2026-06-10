//! `SearchInputRenderer` — visual side of `SearchInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` (the inner
//! painter) plus a search icon at the leading edge and a
//! clear-button at the trailing edge. Escape key clears the
//! value.

use std::any::Any;
use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};
use yororen_ui_core::headless::icon::{IconSource, icon};
use yororen_ui_core::headless::search_input::SearchInputProps;
use yororen_ui_core::headless::text_input::{Escape, TextInputState};
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::text_input::{TextInputElement, start_cursor_blink, wire_input_keyboard};
pub use yororen_ui_core::renderer::search_input::{SearchInputRenderState, SearchInputRenderer};
use yororen_ui_core::renderer::spec::Edges;

pub struct TokenSearchInputRenderer;

impl SearchInputRenderer for TokenSearchInputRenderer {
    fn bg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn focus_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn hover_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn icon_color(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    fn fg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    fn min_height(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.search_input.min_height")
            .unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.search_input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn input_gap(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.search_input.input_gap")
            .unwrap_or(0.0) as f32)
    }
    fn icon_size(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.search_input.icon_size")
            .unwrap_or(0.0) as f32)
    }
}

pub fn arc_search_input<T: SearchInputRenderer + 'static>(r: T) -> Arc<dyn SearchInputRenderer> {
    Arc::new(r)
}
