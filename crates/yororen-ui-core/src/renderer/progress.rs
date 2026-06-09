//! `ProgressBarRenderer` — the visual side of `ProgressBar`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProgressBarRenderState {
    pub indeterminate: bool,
    pub has_custom_height: bool,
}

pub trait ProgressBarRenderer: Any + Send + Sync {
    fn track(&self, state: &ProgressBarRenderState, theme: &Theme) -> Hsla;
    fn fill(&self, state: &ProgressBarRenderState, theme: &Theme) -> Hsla;
    fn height(&self, state: &ProgressBarRenderState, theme: &Theme) -> Pixels;
    fn border_color(&self, state: &ProgressBarRenderState, theme: &Theme) -> Hsla;
    fn border_radius(&self, state: &ProgressBarRenderState, theme: &Theme) -> Pixels;
}
