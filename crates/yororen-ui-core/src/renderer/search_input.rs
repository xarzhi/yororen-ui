//! `SearchInputRenderer` — visual side of `SearchInput`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SearchInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait SearchInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn icon_color(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn input_gap(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn icon_size(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
}
