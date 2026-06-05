//! `SwitchRenderer` — the visual side of `Switch`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SwitchRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
}

pub trait SwitchRenderer: Any + Send + Sync {
    fn track_w(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn track_h(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn knob_size(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &SwitchRenderState, theme: &Theme) -> f32;
}

pub struct TokenSwitchRenderer;

impl SwitchRenderer for TokenSwitchRenderer {
    fn track_w(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.track_w
    }
    fn track_h(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.track_h
    }
    fn knob_size(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.knob_size
    }
    fn padding(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.padding
    }

    fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        let accent = theme.action.primary.bg;
        if state.disabled {
            theme.surface.sunken
        } else if state.checked {
            accent
        } else {
            theme.surface.hover
        }
    }
    fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.border.muted
        } else {
            theme.border.default
        }
    }
    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.hover_bg
        } else {
            theme.surface.base
        }
    }
    fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else if state.checked {
            theme.action.primary.fg
        } else {
            theme.content.primary
        }
    }
    fn focus_color(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &SwitchRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_switch<T: SwitchRenderer + 'static>(r: T) -> Arc<dyn SwitchRenderer> {
    Arc::new(r)
}
