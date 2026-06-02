//! `SkeletonRenderer` — visual side of `SkeletonLine` / `SkeletonBlock`.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SkeletonRenderState {
    /// `true` for `SkeletonBlock`; `false` for `SkeletonLine`.
    pub block: bool,
    /// `true` if the block was configured with `.rounded(false)`.
    pub block_sharp: bool,
}

pub trait SkeletonRenderer: Send + Sync {
    fn bg(&self, state: &SkeletonRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenSkeletonRenderer;

impl SkeletonRenderer for TokenSkeletonRenderer {
    fn bg(&self, _state: &SkeletonRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }

    fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block {
            theme.tokens.control.skeleton.block_min_h
        } else {
            theme.tokens.control.skeleton.line_h
        }
    }

    fn border_radius(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block && !state.block_sharp {
            theme.tokens.radii.md
        } else {
            theme.tokens.radii.pill
        }
    }
}

pub fn arc_skeleton<T: SkeletonRenderer + 'static>(r: T) -> Arc<dyn SkeletonRenderer> {
    Arc::new(r)
}
