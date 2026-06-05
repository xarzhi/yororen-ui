//! `CheckboxRenderer` — the visual side of `Checkbox`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct CheckboxRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
}

pub trait CheckboxRenderer: Any + Send + Sync {
    fn box_size(&self, state: &CheckboxRenderState, theme: &Theme) -> Pixels;
    fn check_size(&self, state: &CheckboxRenderState, theme: &Theme) -> Pixels;
    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn check_fg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &CheckboxRenderState, theme: &Theme) -> f32;
}

pub struct TokenCheckboxRenderer;

impl CheckboxRenderer for TokenCheckboxRenderer {
    fn box_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.box_size
    }
    fn check_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.check_size
    }
    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else if state.checked {
            theme.action.primary.bg
        } else {
            theme.surface.base
        }
    }
    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.bg
        } else {
            theme.border.default
        }
    }
    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.hover_bg
        } else {
            theme.surface.hover
        }
    }
    fn check_fg(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.fg
    }
    fn focus_color(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &CheckboxRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_checkbox<T: CheckboxRenderer + 'static>(r: T) -> Arc<dyn CheckboxRenderer> {
    Arc::new(r)
}
