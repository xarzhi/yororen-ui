//! `DisclosureRenderer` — visual side of `Disclosure`.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct DisclosureRenderState {
    pub open: bool,
}

pub trait DisclosureRenderer: Send + Sync {
    fn trigger_bg(&self, state: &DisclosureRenderState, theme: &Theme) -> Hsla;
    fn trigger_fg(&self, state: &DisclosureRenderState, theme: &Theme) -> Hsla;
    fn trigger_hover_bg(&self, state: &DisclosureRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &DisclosureRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &DisclosureRenderState, theme: &Theme) -> Pixels;
    fn chevron_rotation(&self, state: &DisclosureRenderState, theme: &Theme) -> f32;
    fn body_padding(&self, state: &DisclosureRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenDisclosureRenderer;

impl DisclosureRenderer for TokenDisclosureRenderer {
    fn trigger_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.bg
    }
    fn trigger_fg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.fg
    }
    fn trigger_hover_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.hover_bg
    }
    fn min_height(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn border_radius(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn chevron_rotation(&self, state: &DisclosureRenderState, _theme: &Theme) -> f32 {
        if state.open { 90.0 } else { 0.0 }
    }
    fn body_padding(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_md
    }
}

pub fn arc_disclosure<T: DisclosureRenderer + 'static>(r: T) -> Arc<dyn DisclosureRenderer> {
    Arc::new(r)
}
