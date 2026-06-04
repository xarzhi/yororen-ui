//! `TooltipRenderer` — the visual side of `Tooltip`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

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
        theme.action.neutral.bg
    }

    fn fg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.fg
    }

    fn padding(&self, _state: &TooltipRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(theme.tokens.spacing.inset_md, theme.tokens.spacing.inset_sm)
    }

    fn font_size(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        theme.tokens.typography.font_size_sm
    }

    fn border_radius(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.sm
    }
}

pub fn arc_tooltip<T: TooltipRenderer + 'static>(r: T) -> Arc<dyn TooltipRenderer> {
    Arc::new(r)
}
