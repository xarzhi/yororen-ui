//! `TokenDropdownMenuRenderer` — default `DropdownMenuRenderer` impl.
//!
//! Returns a styled shell (bg, border, radius, shadow, padding).
//! The caller supplies the trigger and dropdown body as children
//! after `.render(cx)`.

use std::sync::Arc;

use gpui::{App, Div, Hsla, Pixels, Styled, div};

use yororen_ui_core::headless::dropdown_menu::DropdownMenuProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::dropdown_menu::{DropdownMenuRenderState, DropdownMenuRenderer};

pub struct TokenDropdownMenuRenderer;

impl TokenDropdownMenuRenderer {
    pub fn bg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn border_radius(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
    }
    pub fn padding(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(4.0) as f32)
    }
    pub fn shadow_alpha(&self, _state: &DropdownMenuRenderState, _theme: &Theme) -> f32 {
        0.12
    }
}

impl DropdownMenuRenderer for TokenDropdownMenuRenderer {
    fn compose(&self, props: &DropdownMenuProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DropdownMenuRenderState {
            open: props.state.read(cx).is_open(),
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let r = self.border_radius(&state, theme);
        let pad = self.padding(&state, theme);
        let alpha = self.shadow_alpha(&state, theme);
        div()
            .bg(bg)
            .border_1()
            .border_color(border)
            .rounded(r)
            .p(pad)
            .shadow(vec![gpui::BoxShadow {
                color: gpui::hsla(0.0, 0.0, 0.0, alpha),
                blur_radius: gpui::px(12.0),
                spread_radius: gpui::px(0.0),
                offset: gpui::Point { x: gpui::px(0.0), y: gpui::px(4.0) },
                
            }])
    }
}

pub fn arc_dropdown_menu<T: DropdownMenuRenderer + 'static>(r: T) -> Arc<dyn DropdownMenuRenderer> {
    Arc::new(r)
}
