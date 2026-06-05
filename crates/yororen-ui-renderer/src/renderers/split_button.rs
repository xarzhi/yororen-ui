//! `SplitButtonRenderer` — visual side of `SplitButton`.

use std::any::Any;
use std::sync::Arc;

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

pub struct TokenSplitButtonRenderer;

impl SplitButtonRenderer for TokenSplitButtonRenderer {
    fn primary_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.bg
    }
    fn primary_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.fg
    }
    fn chevron_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.bg
    }
    fn chevron_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.fg
    }
    fn chevron_hover_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.hover_bg
    }
    fn min_height(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.split_button.min_height
    }
    fn border_radius(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn gap(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.split_button.separator_w
    }
}

pub fn arc_split_button<T: SplitButtonRenderer + 'static>(r: T) -> Arc<dyn SplitButtonRenderer> {
    Arc::new(r)
}
