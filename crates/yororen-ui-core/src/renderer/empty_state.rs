//! `EmptyStateRenderer` — visual side of `EmptyState`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct EmptyStateRenderState {}

pub trait EmptyStateRenderer: Any + Send + Sync {
    fn icon_color(&self, state: &EmptyStateRenderState, theme: &Theme) -> Hsla;
    fn title_color(&self, state: &EmptyStateRenderState, theme: &Theme) -> Hsla;
    fn body_color(&self, state: &EmptyStateRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &EmptyStateRenderState, theme: &Theme) -> Edges<Pixels>;
    fn icon_size(&self, state: &EmptyStateRenderState, theme: &Theme) -> Pixels;
    fn gap(&self, state: &EmptyStateRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenEmptyStateRenderer;

impl EmptyStateRenderer for TokenEmptyStateRenderer {
    fn icon_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn title_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.content.secondary
    }
    fn body_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn padding(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(theme.tokens.spacing.inset_lg)
    }
    fn icon_size(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_xl
    }
    fn gap(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_sm
    }
}

pub fn arc_empty_state<T: EmptyStateRenderer + 'static>(r: T) -> Arc<dyn EmptyStateRenderer> {
    Arc::new(r)
}
