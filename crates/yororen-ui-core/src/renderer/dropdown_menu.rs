//! `DropdownMenuRenderer` — visual side of `DropdownMenu`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct DropdownMenuRenderState {
    pub open: bool,
}

pub trait DropdownMenuRenderer: Any + Send + Sync {
    fn trigger_bg(&self, state: &DropdownMenuRenderState, theme: &Theme) -> Hsla;
    fn trigger_hover_bg(&self, state: &DropdownMenuRenderState, theme: &Theme) -> Hsla;
    fn trigger_fg(&self, state: &DropdownMenuRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &DropdownMenuRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &DropdownMenuRenderState, theme: &Theme) -> Pixels;
    fn chevron_rotation(&self, state: &DropdownMenuRenderState, theme: &Theme) -> f32;
}
