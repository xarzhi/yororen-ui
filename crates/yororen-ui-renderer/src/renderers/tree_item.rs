//! `TreeItemRenderer` — visual side of `TreeItem`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
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

pub struct TokenTreeItemRenderer;

impl TreeItemRenderer for TokenTreeItemRenderer {
    fn bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn hover_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn selected_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.bg
    }
    fn fg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.action.primary.fg
        } else {
            theme.content.primary
        }
    }
    fn indent(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        let step: f32 = theme.tokens.spacing.inset_md.into();
        let step: f32 = step.max(12.0);
        gpui::px(state.depth as f32 * step)
    }
    fn padding(&self, _state: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(theme.tokens.spacing.inset_sm, theme.tokens.spacing.inset_xs)
    }
    fn min_height(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.tree_item.min_height
    }
    fn chevron_size(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.list_item.chevron_size
    }
}

pub fn arc_tree_item<T: TreeItemRenderer + 'static>(r: T) -> Arc<dyn TreeItemRenderer> {
    Arc::new(r)
}
