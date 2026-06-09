//! `ModalRenderer` — visual side of `Modal`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
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
