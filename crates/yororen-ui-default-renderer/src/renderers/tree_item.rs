//! `TreeItemRenderer` — visual side of `TreeItem`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::renderer::spec::Edges;
pub use yororen_ui_core::renderer::tree_item::{TreeItemRenderState, TreeItemRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenTreeItemRenderer;

impl TreeItemRenderer for TokenTreeItemRenderer {
    fn bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn hover_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    fn selected_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    fn fg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    fn indent(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        let step = theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32;
        let step = step.max(12.0);
        gpui::px(state.depth as f32 * step)
    }
    fn padding(&self, _state: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    fn min_height(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.tree_item.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    fn chevron_size(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.list_item.chevron_size")
                .unwrap_or(0.0) as f32,
        )
    }
}

pub fn arc_tree_item<T: TreeItemRenderer + 'static>(r: T) -> Arc<dyn TreeItemRenderer> {
    Arc::new(r)
}
