//! `ProgressBarRenderer` — the visual side of `ProgressBar`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProgressBarRenderState {
    pub indeterminate: bool,
    /// `true` if user supplied a non-zero `.height(...)`.
    pub has_custom_height: bool,
}

pub trait ProgressBarRenderer: Any + Send + Sync {
    fn track(&self, state: &ProgressBarRenderState, theme: &Theme) -> Hsla;
    fn fill(&self, state: &ProgressBarRenderState, theme: &Theme) -> Hsla;
    fn height(&self, state: &ProgressBarRenderState, theme: &Theme) -> Pixels;
    fn border_color(&self, state: &ProgressBarRenderState, theme: &Theme) -> Hsla;
    fn border_radius(&self, state: &ProgressBarRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenProgressBarRenderer;

impl ProgressBarRenderer for TokenProgressBarRenderer {
    fn track(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }

    fn fill(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.bg
    }

    fn height(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.progress.bar_default_h
    }

    fn border_color(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }

    fn border_radius(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.pill
    }
}

pub fn arc_progress_bar<T: ProgressBarRenderer + 'static>(r: T) -> Arc<dyn ProgressBarRenderer> {
    Arc::new(r)
}
