//! `CardRenderer` — visual side of `Card`.

use std::any::Any;

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
