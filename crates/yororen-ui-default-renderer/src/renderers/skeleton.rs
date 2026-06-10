//! `TokenSkeletonRenderer` — default `SkeletonRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, Pixels, Styled, div};

use yororen_ui_core::headless::skeleton::SkeletonProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::skeleton::{SkeletonRenderState, SkeletonRenderer};

pub struct TokenSkeletonRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenSkeletonRenderer {
    pub fn bg(&self, _state: &SkeletonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block {
            gpui::px(
                theme
                    .get_number("tokens.control.skeleton.block_min_h")
                    .unwrap_or(48.0) as f32,
            )
        } else {
            gpui::px(
                theme
                    .get_number("tokens.control.skeleton.line_h")
                    .unwrap_or(16.0) as f32,
            )
        }
    }
    pub fn border_radius(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block && !state.block_sharp {
            gpui::px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
        } else if !state.block {
            gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(9999.0) as f32)
        } else {
            gpui::px(0.0)
        }
    }
}

impl SkeletonRenderer for TokenSkeletonRenderer {
    fn compose(&self, props: &SkeletonProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = SkeletonRenderState {
            block: props.block,
            block_sharp: props.block_sharp,
            rounded: props.rounded,
        };
        let bg = self.bg(&state, theme);
        let min_h = self.min_height(&state, theme);
        let radius = self.border_radius(&state, theme);
        div().bg(bg).min_h(min_h).rounded(radius)
    }
}

pub fn arc_skeleton<T: SkeletonRenderer + 'static>(r: T) -> Arc<dyn SkeletonRenderer> {
    Arc::new(r)
}
