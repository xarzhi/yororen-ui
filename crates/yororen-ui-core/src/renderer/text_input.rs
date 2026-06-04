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
        } else {
            theme.surface.base
        }
    }
    fn border(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
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
    fn disabled_opacity(&self, _state: &TextInputRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

pub fn arc_text_input<T: TextInputRenderer + 'static>(r: T) -> Arc<dyn TextInputRenderer> {
    Arc::new(r)
}
