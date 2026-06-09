//! `SelectRenderer` — visual side of `Select`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SelectRenderState {
    pub open: bool,
    pub disabled: bool,
    pub has_value: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait SelectRenderer: Any + Send + Sync {
    fn bg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn hint_color(&self, state: &SelectRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SelectRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &SelectRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &SelectRenderState, theme: &Theme) -> Pixels;
    fn chevron_rotation(&self, state: &SelectRenderState, theme: &Theme) -> f32;
}
