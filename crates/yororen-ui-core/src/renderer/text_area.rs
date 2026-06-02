//! `TextAreaRenderer` — visual side of `TextArea`.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextAreaRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
}

pub trait TextAreaRenderer: Send + Sync {
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
    fn bg(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn border(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn text_color(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
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
