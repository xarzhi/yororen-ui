//! `RadioRenderer` — the visual side of `Radio`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct RadioRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
    pub custom_tone: Option<Hsla>,
}

pub trait RadioRenderer: Any + Send + Sync {
    fn ring_size(&self, state: &RadioRenderState, theme: &Theme) -> Pixels;
    fn dot_size(&self, state: &RadioRenderState, theme: &Theme) -> Pixels;
    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_hover_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_active_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &RadioRenderState, theme: &Theme) -> f32;
}
