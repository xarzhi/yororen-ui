//! `TooltipRenderer` — the visual side of `Tooltip`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TooltipRenderState {
    pub has_custom_bg: bool,
    pub has_custom_fg: bool,
}

pub trait TooltipRenderer: Any + Send + Sync {
    fn bg(&self, state: &TooltipRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &TooltipRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &TooltipRenderState, theme: &Theme) -> Edges<Pixels>;
    fn font_size(&self, state: &TooltipRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &TooltipRenderState, theme: &Theme) -> Pixels;
}
