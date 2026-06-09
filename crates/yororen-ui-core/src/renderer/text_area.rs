//! `TextAreaRenderer` — visual side of `TextArea`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextAreaRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub has_custom_focus_border: bool,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextAreaRenderer: Any + Send + Sync {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TextAreaRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &TextAreaRenderState, theme: &Theme) -> Pixels;
}
