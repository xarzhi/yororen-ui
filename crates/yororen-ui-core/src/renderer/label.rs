//! `LabelRenderer` — the visual side of `Label`.

use std::any::Any;

use gpui::{FontWeight, Hsla, SharedString};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct LabelRenderState {
    pub muted: bool,
    pub strong: bool,
    pub mono: bool,
    pub inherit_color: bool,
    pub ellipsis: bool,
    pub wrap: bool,
    pub max_lines: Option<usize>,
}

pub trait LabelRenderer: Any + Send + Sync {
    fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla;
    fn strong_weight(&self, state: &LabelRenderState, theme: &Theme) -> FontWeight;
    fn family_mono(&self, state: &LabelRenderState, theme: &Theme) -> SharedString;
}
