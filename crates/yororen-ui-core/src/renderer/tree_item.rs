//! `TreeItemRenderer` — visual side of `TreeItem`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TreeItemRenderState {
    pub selected: bool,
    pub expanded: bool,
    pub depth: u8,
    pub is_leaf: bool,
}

pub trait TreeItemRenderer: Any + Send + Sync {
    fn bg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla;
    fn hover_bg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla;
    fn selected_bg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla;
    fn indent(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels>;
    fn min_height(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels;
    fn chevron_size(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels;
}
