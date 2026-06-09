//! `PanelRenderer` — the visual side of `Panel`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct PanelRenderState {
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_padding: bool,
}

pub trait PanelRenderer: Any + Send + Sync {
    fn bg(&self, state: &PanelRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &PanelRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &PanelRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &PanelRenderState, theme: &Theme) -> Pixels;
    fn shadow_alpha(&self, state: &PanelRenderState, theme: &Theme) -> f32;
}
