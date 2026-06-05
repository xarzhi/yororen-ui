//! `DropdownMenuRenderer` — visual side of `DropdownMenu`.

use std::any::Any;
use std::sync::Arc;

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

pub struct TokenDropdownMenuRenderer;

impl DropdownMenuRenderer for TokenDropdownMenuRenderer {
    fn trigger_bg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.bg
    }
    fn trigger_hover_bg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.hover_bg
    }
    fn trigger_fg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.fg
    }
    fn min_height(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn border_radius(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn chevron_rotation(&self, state: &DropdownMenuRenderState, _theme: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

pub fn arc_dropdown_menu<T: DropdownMenuRenderer + 'static>(r: T) -> Arc<dyn DropdownMenuRenderer> {
    Arc::new(r)
}
