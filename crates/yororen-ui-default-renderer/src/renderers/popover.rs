//! `TokenPopoverRenderer` — default `PopoverRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, Pixels, Styled, div};

use yororen_ui_core::headless::popover::PopoverProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::popover::{PopoverRenderState, PopoverRenderer};

pub struct TokenPopoverRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenPopoverRenderer {
    pub fn bg(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or_default()
    }
    pub fn border(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    pub fn shadow_alpha(&self, _state: &PopoverRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_2").unwrap_or_default().a
    }
    pub fn border_radius(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn offset(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.popover.offset")
                .unwrap_or(0.0) as f32,
        )
    }
}

impl PopoverRenderer for TokenPopoverRenderer {
    fn compose(&self, _props: &PopoverProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = PopoverRenderState {};
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let r = self.border_radius(&state, theme);
        div().bg(bg).border_color(border).rounded(r)
    }
}

pub fn arc_popover<T: PopoverRenderer + 'static>(r: T) -> Arc<dyn PopoverRenderer> {
    Arc::new(r)
}
