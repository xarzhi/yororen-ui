//! `TokenTreeItemRenderer` — default `TreeItemRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::tree_item::TreeItemProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::tree_item::{TreeItemRenderState, TreeItemRenderer};

pub struct TokenTreeItemRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenTreeItemRenderer {
    pub fn bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn selected_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    pub fn fg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    pub fn indent(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        let step = theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32;
        let step = step.max(12.0);
        gpui::px(state.depth as f32 * step)
    }
    pub fn padding(&self, _state: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    pub fn min_height(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.tree_item.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn chevron_size(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.list_item.chevron_size")
                .unwrap_or(0.0) as f32,
        )
    }
}

impl TreeItemRenderer for TokenTreeItemRenderer {
    fn compose(&self, props: &TreeItemProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TreeItemRenderState {
            selected: props.selected,
            expanded: props.expanded,
            depth: props.depth.min(u8::MAX as usize) as u8,
            is_leaf: !props.has_children,
        };
        let bg = if state.selected {
            self.selected_bg(&state, theme)
        } else {
            self.bg(&state, theme)
        };
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let indent = self.indent(&state, theme);
        div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .pl(indent + pad.left)
            .pr(pad.right)
            .min_h(h)
            .child(props.label.clone())
    }
}

pub fn arc_tree_item<T: TreeItemRenderer + 'static>(r: T) -> Arc<dyn TreeItemRenderer> {
    Arc::new(r)
}
