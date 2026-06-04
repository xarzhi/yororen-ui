//! `TextInputRenderer` — visual side of `TextInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    /// Caller-supplied overrides. When the corresponding
    /// `has_custom_*` is true, the renderer returns this color
    /// instead of the built-in token path.
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn hint_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TextInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TextInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &TextInputRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &TextInputRenderState, theme: &Theme) -> f32;
}

pub struct TokenTextInputRenderer;

impl TextInputRenderer for TokenTextInputRenderer {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else if state.has_custom_bg {
            state.custom_bg.unwrap_or(theme.surface.base)
        } else {
            theme.surface.base
        }
    }
    fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.border.muted
        } else if state.has_custom_border {
            state.custom_border.unwrap_or(theme.border.default)
        } else {
            theme.border.default
        }
    }
    fn focus_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_focus_border {
            state.custom_focus_border.unwrap_or(theme.border.focus)
        } else {
            theme.border.focus
        }
    }
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else if state.custom_text_color.is_some() {
            state.custom_text_color.unwrap()
        } else {
            theme.content.primary
        }
    }
    fn hint_color(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn min_height(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn padding(&self, _state: &TextInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn border_radius(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn disabled_opacity(&self, state: &TextInputRenderState, _theme: &Theme) -> f32 {
        if state.disabled { 0.6 } else { 1.0 }
    }
}

pub fn arc_text_input<T: TextInputRenderer + 'static>(r: T) -> Arc<dyn TextInputRenderer> {
    Arc::new(r)
}
