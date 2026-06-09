//! `TextInputRenderer` — visual side of `TextInput`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn hint_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn cursor_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn selection_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TextInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TextInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &TextInputRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &TextInputRenderState, theme: &Theme) -> f32;
}
