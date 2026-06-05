//! `RadioRenderer` — the visual side of `Radio`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct RadioRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
}

pub trait RadioRenderer: Any + Send + Sync {
    fn ring_size(&self, state: &RadioRenderState, theme: &Theme) -> Pixels;
    fn dot_size(&self, state: &RadioRenderState, theme: &Theme) -> Pixels;
    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_hover_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &RadioRenderState, theme: &Theme) -> f32;
}

pub struct TokenRadioRenderer;

impl RadioRenderer for TokenRadioRenderer {
    fn ring_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.ring_size
    }
    fn dot_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.dot_size
    }
    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.bg
        } else {
            theme.border.default
        }
    }
    fn ring_hover_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn dot_fg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.bg
    }
    fn focus_color(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &RadioRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_radio<T: RadioRenderer + 'static>(r: T) -> Arc<dyn RadioRenderer> {
    Arc::new(r)
}
