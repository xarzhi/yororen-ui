//! `CheckboxRenderer` — the visual side of `Checkbox`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct CheckboxRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
    pub custom_tone: Option<Hsla>,
}

pub trait CheckboxRenderer: Any + Send + Sync {
    fn box_size(&self, state: &CheckboxRenderState, theme: &Theme) -> Pixels;
    fn check_size(&self, state: &CheckboxRenderState, theme: &Theme) -> Pixels;
    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn box_active_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn check_fg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &CheckboxRenderState, theme: &Theme) -> f32;
}
