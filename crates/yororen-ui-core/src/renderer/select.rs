//! `SelectRenderer` — visual side of `Select`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SelectRenderState {
    pub open: bool,
    pub disabled: bool,
    pub has_value: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait SelectRenderer: Any + Send + Sync {
    fn bg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn hint_color(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SelectRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &SelectRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &SelectRenderState, theme: &Theme) -> Pixels;
    fn chevron_rotation(&self, state: &SelectRenderState, theme: &Theme) -> f32;
}

pub struct TokenSelectRenderer;

impl SelectRenderer for TokenSelectRenderer {
    fn bg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            state.custom_bg.unwrap_or(theme.surface.base)
        }
    }
    fn border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.border.muted
        } else {
            state.custom_border.unwrap_or(theme.border.default)
        }
    }
    fn focus_border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        state.custom_focus_border.unwrap_or(theme.border.focus)
    }
    fn fg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else if state.custom_fg.is_some() {
            return state.custom_fg.unwrap();
        } else if state.has_value {
            theme.content.primary
        } else {
            theme.content.tertiary
        }
    }
    fn hint_color(&self, _state: &SelectRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn min_height(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn padding(&self, _state: &SelectRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(theme.tokens.spacing.inset_sm, theme.tokens.spacing.inset_xs)
    }
    fn border_radius(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn chevron_rotation(&self, state: &SelectRenderState, _theme: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

pub fn arc_select<T: SelectRenderer + 'static>(r: T) -> Arc<dyn SelectRenderer> {
    Arc::new(r)
}
