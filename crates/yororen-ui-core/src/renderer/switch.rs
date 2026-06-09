//! `SwitchRenderer` — the visual side of `Switch`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SwitchRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
    pub custom_tone: Option<Hsla>,
}

pub trait SwitchRenderer: Any + Send + Sync {
    fn track_w(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn track_h(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn knob_size(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn track_active_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &SwitchRenderState, theme: &Theme) -> f32;
}
