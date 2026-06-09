//! `BadgeRenderer` — the visual side of `Badge`.

use std::any::Any;

use gpui::{FontWeight, Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct BadgeRenderState {
    pub variant: crate::headless::badge::BadgeVariant,
    pub has_custom_tone: bool,
}

pub trait BadgeRenderer: Any + Send + Sync {
    fn bg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla;
    fn padding_x(&self, state: &BadgeRenderState, theme: &Theme) -> Pixels;
    fn height(&self, state: &BadgeRenderState, theme: &Theme) -> Pixels;
    fn font_size(&self, state: &BadgeRenderState, theme: &Theme) -> Pixels;
    fn font_weight(&self, state: &BadgeRenderState, theme: &Theme) -> FontWeight;
    fn border_radius(&self, state: &BadgeRenderState, theme: &Theme) -> Pixels;
}
