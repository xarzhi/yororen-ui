//! Brutalist list renderers: `ListItem`, `TreeItem`, `Form`.

use gpui::{Hsla, Pixels, px};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS};

// =====================================================================
// ListItem
// =====================================================================

pub use yororen_ui_core::renderer::list_item::{ListItemRenderState, ListItemRenderer};

pub struct BrutalListItemRenderer;

impl ListItemRenderer for BrutalListItemRenderer {
    fn bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    fn hover_bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    fn selected_bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }
    fn padding(&self, _: &ListItemRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.list_item.padding")
            .unwrap_or(10.0) as f32;
        Edges::symmetric(px(h), px(h / 2.0))
    }
    fn min_height(&self, _: &ListItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.list_item.min_height")
            .unwrap_or(36.0) as f32)
    }
    fn border_radius(&self, _: &ListItemRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// TreeItem
// =====================================================================

pub use yororen_ui_core::renderer::tree_item::{TreeItemRenderState, TreeItemRenderer};

pub struct BrutalTreeItemRenderer;

impl TreeItemRenderer for BrutalTreeItemRenderer {
    fn bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    fn hover_bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    fn selected_bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn fg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }
    fn indent(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        let step = theme
            .get_number("tokens.control.tree_item.indent_step")
            .unwrap_or(16.0) as f32;
        px(state.depth as f32 * step)
    }
    fn padding(&self, _: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.tree_item.padding")
            .unwrap_or(8.0) as f32;
        Edges::symmetric(px(p), px(p / 2.0))
    }
    fn min_height(&self, _: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tree_item.min_height")
            .unwrap_or(32.0) as f32)
    }
    fn chevron_size(&self, _: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.list_item.chevron_size")
            .unwrap_or(16.0) as f32)
    }
}

// =====================================================================
// Form
// =====================================================================

pub use yororen_ui_core::renderer::form::{FormRenderState, FormRenderer};

pub struct BrutalFormRenderer;

impl FormRenderer for BrutalFormRenderer {
    fn gap(&self, _: &FormRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.form.gap").unwrap_or(12.0) as f32)
    }
    fn label_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    fn error_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("status.error.fg").unwrap_or(BRUTAL_BORDER)
    }
    fn helper_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
    }
}
