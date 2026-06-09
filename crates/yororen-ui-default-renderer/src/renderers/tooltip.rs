//! `TooltipRenderer` — the visual side of `Tooltip`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TooltipRenderState {
    pub has_custom_bg: bool,
    pub has_custom_fg: bool,
}

pub trait TooltipRenderer: Any + Send + Sync {
    fn bg(&self, state: &TooltipRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &TooltipRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &TooltipRenderState, theme: &Theme) -> Edges<Pixels>;
    fn font_size(&self, state: &TooltipRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &TooltipRenderState, theme: &Theme) -> Pixels;
}

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
