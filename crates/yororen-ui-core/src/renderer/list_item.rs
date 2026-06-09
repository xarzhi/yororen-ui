//! `ListItemRenderer` — visual side of `ListItem`.

use std::any::Any;

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
