//! `SelectRenderer` — visual side of `Select`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::select::{SelectRenderState, SelectRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub struct TokenSelectRenderer;

impl SelectRenderer for TokenSelectRenderer {
    fn bg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    fn border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    fn focus_border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    fn fg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.custom_fg.is_some() {
            state.custom_fg.unwrap()
        } else if state.has_value {
            theme.get_color("content.primary").unwrap_or_default()
        } else {
            theme.get_color("content.tertiary").unwrap_or_default()
        }
    }
    fn hint_color(&self, _state: &SelectRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    fn min_height(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    fn padding(&self, _state: &SelectRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn chevron_rotation(&self, state: &SelectRenderState, _theme: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

pub fn arc_select<T: SelectRenderer + 'static>(r: T) -> Arc<dyn SelectRenderer> {
    Arc::new(r)
}
