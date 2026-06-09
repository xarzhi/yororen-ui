//! `SkeletonRenderer` — visual side of `SkeletonLine` / `SkeletonBlock`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SkeletonRenderState {
    pub block: bool,
    pub block_sharp: bool,
}

pub trait SkeletonRenderer: Any + Send + Sync {
    fn bg(&self, state: &SkeletonRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels;
}
