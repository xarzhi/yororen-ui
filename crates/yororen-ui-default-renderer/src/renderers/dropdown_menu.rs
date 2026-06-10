//! `TokenDropdownMenuRenderer` — default `DropdownMenuRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::dropdown_menu::DropdownMenuProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::dropdown_menu::{DropdownMenuRenderState, DropdownMenuRenderer};

pub struct TokenDropdownMenuRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenDropdownMenuRenderer {
    pub fn trigger_bg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn trigger_hover_bg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    pub fn trigger_fg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn chevron_rotation(&self, state: &DropdownMenuRenderState, _theme: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

impl DropdownMenuRenderer for TokenDropdownMenuRenderer {
    fn compose(&self, props: &DropdownMenuProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DropdownMenuRenderState {
            open: props.state.read(cx).is_open(),
        };
        let bg = self.trigger_bg(&state, theme);
        let fg = self.trigger_fg(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .min_h(h)
            .rounded(r)
            .child("▼")
    }
}

pub fn arc_dropdown_menu<T: DropdownMenuRenderer + 'static>(r: T) -> Arc<dyn DropdownMenuRenderer> {
    Arc::new(r)
}
