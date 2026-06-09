//! `ToastRenderer` — visual side of `Toast`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ToastRenderState {
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
