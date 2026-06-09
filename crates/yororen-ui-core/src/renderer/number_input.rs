//! `NumberInputRenderer` — visual side of `NumberInput`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct NumberInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait NumberInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn stepper_button_size(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
}
