//! `SkeletonRenderer` — visual side of `SkeletonLine` / `SkeletonBlock`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SkeletonRenderState {
    /// `true` for `SkeletonBlock`; `false` for `SkeletonLine`.
    pub block: bool,
    /// `true` if the block was configured with `.rounded(false)`.
    pub block_sharp: bool,
}

pub trait SkeletonRenderer: Any + Send + Sync {
    fn bg(&self, state: &SkeletonRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenSkeletonRenderer;

impl SkeletonRenderer for TokenSkeletonRenderer {
    fn bg(&self, _state: &SkeletonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }

    fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block {
            gpui::px(
                theme
                    .get_number("tokens.control.skeleton.block_min_h")
                    .unwrap_or(0.0) as f32,
            )
        } else {
            gpui::px(
                theme
                    .get_number("tokens.control.skeleton.line_h")
                    .unwrap_or(0.0) as f32,
            )
        }
    }

    fn border_radius(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block && !state.block_sharp {
            gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
        } else {
            gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
        }
    }
}

pub fn arc_skeleton<T: SkeletonRenderer + 'static>(r: T) -> Arc<dyn SkeletonRenderer> {
    Arc::new(r)
}
