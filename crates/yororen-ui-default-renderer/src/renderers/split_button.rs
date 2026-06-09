//! `SplitButtonRenderer` — visual side of `SplitButton`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

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
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    fn primary_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.fg").unwrap_or_default()
    }
    fn chevron_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    fn chevron_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    fn chevron_hover_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    fn min_height(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.split_button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    fn border_radius(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn gap(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.split_button.separator_w")
                .unwrap_or(0.0) as f32,
        )
    }
}

pub fn arc_split_button<T: SplitButtonRenderer + 'static>(r: T) -> Arc<dyn SplitButtonRenderer> {
    Arc::new(r)
}
