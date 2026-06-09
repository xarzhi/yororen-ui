//! `TagRenderer` — the visual side of `Tag`.

use std::any::Any;

use gpui::{FontWeight, Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TagRenderState {
    pub selected: bool,
    pub has_custom_tone: bool,
    pub closable: bool,
}

pub trait TagRenderer: Any + Send + Sync {
    fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn padding_x(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn font_size(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn font_weight(&self, state: &TagRenderState, theme: &Theme) -> FontWeight;
    fn border_radius(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn close_size(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn close_hover_bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla;
}
