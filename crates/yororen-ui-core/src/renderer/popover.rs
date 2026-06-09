//! `PopoverRenderer` — visual side of `Popover`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct PopoverRenderState {}

pub trait PopoverRenderer: Any + Send + Sync {
    fn bg(&self, state: &PopoverRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &PopoverRenderState, theme: &Theme) -> Hsla;
    fn shadow_alpha(&self, state: &PopoverRenderState, theme: &Theme) -> f32;
    fn border_radius(&self, state: &PopoverRenderState, theme: &Theme) -> Pixels;
    fn offset(&self, state: &PopoverRenderState, theme: &Theme) -> Pixels;
}
