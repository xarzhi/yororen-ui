//! `ModalRenderer` — visual side of `Modal`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ModalRenderState {}

pub trait ModalRenderer: Any + Send + Sync {
    fn scrim(&self, state: &ModalRenderState, theme: &Theme) -> Hsla;
    fn panel_bg(&self, state: &ModalRenderState, theme: &Theme) -> Hsla;
    fn panel_border(&self, state: &ModalRenderState, theme: &Theme) -> Hsla;
    fn panel_padding(&self, state: &ModalRenderState, theme: &Theme) -> Edges<Pixels>;
    fn panel_border_radius(&self, state: &ModalRenderState, theme: &Theme) -> Pixels;
    fn panel_shadow_alpha(&self, state: &ModalRenderState, theme: &Theme) -> f32;
}

pub struct TokenModalRenderer;

impl ModalRenderer for TokenModalRenderer {
    fn scrim(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        let mut c = theme.shadow.elevation_2;
        c.a = 0.5;
        c
    }
    fn panel_bg(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn panel_border(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }
    fn panel_padding(&self, _state: &ModalRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(theme.tokens.spacing.inset_lg)
    }
    fn panel_border_radius(&self, _state: &ModalRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.lg
    }
    fn panel_shadow_alpha(&self, _state: &ModalRenderState, theme: &Theme) -> f32 {
        theme.shadow.elevation_2.a
    }
}

pub fn arc_modal<T: ModalRenderer + 'static>(r: T) -> Arc<dyn ModalRenderer> {
    Arc::new(r)
}
