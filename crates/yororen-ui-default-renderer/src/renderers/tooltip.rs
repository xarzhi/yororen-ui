//! `TooltipRenderer` — the visual side of `Tooltip`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::renderer::spec::Edges;
pub use yororen_ui_core::renderer::tooltip::{TooltipRenderState, TooltipRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenTooltipRenderer;

impl TooltipRenderer for TokenTooltipRenderer {
    fn bg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }

    fn fg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }

    fn padding(&self, _state: &TooltipRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
        )
    }

    fn font_size(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_sm")
                .unwrap_or(0.0) as f32,
        )
    }

    fn border_radius(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
}

pub fn arc_tooltip<T: TooltipRenderer + 'static>(r: T) -> Arc<dyn TooltipRenderer> {
    Arc::new(r)
}
