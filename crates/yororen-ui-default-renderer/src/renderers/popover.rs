//! `PopoverRenderer` — visual side of `Popover`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

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
        theme.get_color("surface.raised").unwrap_or_default()
    }
    fn border(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn shadow_alpha(&self, _state: &PopoverRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_2").unwrap_or_default().a
    }
    fn border_radius(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn offset(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.popover.offset")
                .unwrap_or(0.0) as f32,
        )
    }
}

pub fn arc_popover<T: PopoverRenderer + 'static>(r: T) -> Arc<dyn PopoverRenderer> {
    Arc::new(r)
}
