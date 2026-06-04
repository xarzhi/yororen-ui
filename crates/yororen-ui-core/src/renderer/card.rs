//! `CardRenderer` — visual side of `Card`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct CardRenderState {
    pub has_custom_bg: bool,
}

pub trait CardRenderer: Any + Send + Sync {
    fn bg(&self, state: &CardRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &CardRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &CardRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &CardRenderState, theme: &Theme) -> Pixels;
    fn shadow_alpha(&self, state: &CardRenderState, theme: &Theme) -> f32;
}

pub struct TokenCardRenderer;

impl CardRenderer for TokenCardRenderer {
    fn bg(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn border(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }
    fn padding(&self, _state: &CardRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(theme.tokens.spacing.inset_md)
    }
    fn border_radius(&self, _state: &CardRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.lg
    }
    fn shadow_alpha(&self, _state: &CardRenderState, theme: &Theme) -> f32 {
        theme.shadow.elevation_1.a
    }
}

pub fn arc_card<T: CardRenderer + 'static>(r: T) -> Arc<dyn CardRenderer> {
    Arc::new(r)
}
