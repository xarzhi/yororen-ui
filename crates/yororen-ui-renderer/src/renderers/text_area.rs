//! `TextAreaRenderer` — visual side of `TextArea`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextAreaRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextAreaRenderer: Any + Send + Sync {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TextAreaRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &TextAreaRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenTextAreaRenderer;

impl TextAreaRenderer for TokenTextAreaRenderer {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            state.custom_bg.unwrap_or(theme.surface.base)
        }
    }
    fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.border.muted
        } else {
            state.custom_border.unwrap_or(theme.border.default)
        }
    }
    fn focus_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        state.custom_focus_border.unwrap_or(theme.border.focus)
    }
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else {
            state.custom_text_color.unwrap_or(theme.content.primary)
        }
    }
    fn min_height(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.text_area_min_h
    }
    fn padding(&self, _state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(theme.tokens.control.input.vertical_padding)
    }
    fn border_radius(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
}

pub fn arc_text_area<T: TextAreaRenderer + 'static>(r: T) -> Arc<dyn TextAreaRenderer> {
    Arc::new(r)
}
