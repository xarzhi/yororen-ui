//! `EmptyStateRenderer` — visual side of `EmptyState`.

use std::any::Any;

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
