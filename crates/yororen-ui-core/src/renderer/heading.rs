//! `HeadingRenderer` — the visual side of `Heading`.

use std::sync::Arc;

use gpui::{FontWeight, Hsla, Pixels};

use crate::component::HeadingLevel;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug)]
pub struct HeadingRenderState {
    pub level: HeadingLevel,
}

pub trait HeadingRenderer: Send + Sync {
    fn size(&self, state: &HeadingRenderState, theme: &Theme) -> Pixels;
    fn weight(&self, state: &HeadingRenderState, theme: &Theme) -> FontWeight;
    fn color(&self, state: &HeadingRenderState, theme: &Theme) -> Hsla;
}

pub struct TokenHeadingRenderer;

impl HeadingRenderer for TokenHeadingRenderer {
    fn size(&self, state: &HeadingRenderState, theme: &Theme) -> Pixels {
        match state.level {
            HeadingLevel::H1 => theme.tokens.typography.font_size_2xl,
            HeadingLevel::H2 => theme.tokens.typography.font_size_xl,
            HeadingLevel::H3 => theme.tokens.typography.font_size_lg,
        }
    }

    fn weight(&self, state: &HeadingRenderState, theme: &Theme) -> FontWeight {
        match state.level {
            HeadingLevel::H1 => theme.tokens.typography.weight_bold,
            _ => theme.tokens.typography.weight_semibold,
        }
    }

    fn color(&self, _state: &HeadingRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
}

pub fn arc_heading<T: HeadingRenderer + 'static>(r: T) -> Arc<dyn HeadingRenderer> {
    Arc::new(r)
}
