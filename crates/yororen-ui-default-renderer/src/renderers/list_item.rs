//! `ListItemRenderer` — visual side of `ListItem`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::list_item::{ListItemRenderState, ListItemRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub struct TokenListItemRenderer;

impl ListItemRenderer for TokenListItemRenderer {
    fn bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn hover_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    fn selected_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    fn padding(&self, _state: &ListItemRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    fn min_height(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.list_item.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    fn border_radius(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
}

pub fn arc_list_item<T: ListItemRenderer + 'static>(r: T) -> Arc<dyn ListItemRenderer> {
    Arc::new(r)
}
