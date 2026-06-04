//! `ListItemRenderer` — visual side of `ListItem`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ListItemRenderState {
    pub selected: bool,
    pub disabled: bool,
    pub hovered: bool,
}

pub trait ListItemRenderer: Any + Send + Sync {
    fn bg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla;
    fn hover_bg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla;
    fn selected_bg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &ListItemRenderState, theme: &Theme) -> Edges<Pixels>;
    fn min_height(&self, state: &ListItemRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &ListItemRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenListItemRenderer;

impl ListItemRenderer for TokenListItemRenderer {
    fn bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn hover_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn selected_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.bg
    }
    fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else if state.selected {
            theme.action.primary.fg
        } else {
            theme.content.primary
        }
    }
    fn padding(&self, _state: &ListItemRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(theme.tokens.spacing.inset_sm, theme.tokens.spacing.inset_xs)
    }
    fn min_height(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.list_item.min_height
    }
    fn border_radius(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.sm
    }
}

pub fn arc_list_item<T: ListItemRenderer + 'static>(r: T) -> Arc<dyn ListItemRenderer> {
    Arc::new(r)
}
