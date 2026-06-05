//! `PopoverRenderer` — visual side of `Popover`.

use std::any::Any;
use std::sync::Arc;

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

pub struct TokenPopoverRenderer;

impl PopoverRenderer for TokenPopoverRenderer {
    fn bg(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn border(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }
    fn shadow_alpha(&self, _state: &PopoverRenderState, theme: &Theme) -> f32 {
        theme.shadow.elevation_2.a
    }
    fn border_radius(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn offset(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.popover.offset
    }
}

pub fn arc_popover<T: PopoverRenderer + 'static>(r: T) -> Arc<dyn PopoverRenderer> {
    Arc::new(r)
}
