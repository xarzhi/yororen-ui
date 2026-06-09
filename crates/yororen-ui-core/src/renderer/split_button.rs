//! `SplitButtonRenderer` — visual side of `SplitButton`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SplitButtonRenderState {
    pub open: bool,
    pub disabled: bool,
}

pub trait SplitButtonRenderer: Any + Send + Sync {
    fn primary_bg(&self, state: &SplitButtonRenderState, theme: &Theme) -> Hsla;
    fn primary_fg(&self, state: &SplitButtonRenderState, theme: &Theme) -> Hsla;
    fn chevron_bg(&self, state: &SplitButtonRenderState, theme: &Theme) -> Hsla;
    fn chevron_fg(&self, state: &SplitButtonRenderState, theme: &Theme) -> Hsla;
    fn chevron_hover_bg(&self, state: &SplitButtonRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SplitButtonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &SplitButtonRenderState, theme: &Theme) -> Pixels;
    fn gap(&self, state: &SplitButtonRenderState, theme: &Theme) -> Pixels;
}
