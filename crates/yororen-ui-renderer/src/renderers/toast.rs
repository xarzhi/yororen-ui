//! `ToastRenderer` — visual side of `Toast`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ToastRenderState {
    /// `true` if the toast has a custom override color.
    pub has_custom_color: bool,
}

pub trait ToastRenderer: Any + Send + Sync {
    fn bg(&self, state: &ToastRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ToastRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &ToastRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &ToastRenderState, theme: &Theme) -> Pixels;
    fn border(&self, state: &ToastRenderState, theme: &Theme) -> Hsla;
    fn shadow_alpha(&self, state: &ToastRenderState, theme: &Theme) -> f32;
}

pub struct TokenToastRenderer;

impl ToastRenderer for TokenToastRenderer {
    fn bg(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn fg(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn padding(&self, _state: &ToastRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(theme.tokens.spacing.inset_md, theme.tokens.spacing.inset_sm)
    }
    fn border_radius(&self, _state: &ToastRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn border(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }
    fn shadow_alpha(&self, _state: &ToastRenderState, theme: &Theme) -> f32 {
        theme.shadow.elevation_2.a
    }
}

pub fn arc_toast<T: ToastRenderer + 'static>(r: T) -> Arc<dyn ToastRenderer> {
    Arc::new(r)
}
