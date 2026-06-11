//! `TokenMenuRenderer` — default `MenuRenderer` impl.
//!
//! Paints a rounded panel with a border and subtle shadow — the
//! typical dropdown / context-menu shell.

use std::sync::Arc;

use gpui::{InteractiveElement, App, Div, Hsla, Pixels, Stateful, Styled, div};

use yororen_ui_core::headless::menu::MenuProps;
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::menu::{MenuRenderState, MenuRenderer};

pub struct TokenMenuRenderer;

impl TokenMenuRenderer {
    pub fn bg(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn border_radius(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
    }
    pub fn padding(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(4.0) as f32)
    }
    pub fn shadow_alpha(&self, _state: &MenuRenderState, _theme: &Theme) -> f32 {
        0.12
    }
}

impl MenuRenderer for TokenMenuRenderer {
    fn compose(&self, props: &MenuProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = MenuRenderState {};
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let radius = self.border_radius(&state, theme);
        let pad = self.padding(&state, theme);
        let alpha = self.shadow_alpha(&state, theme);

        div()
            .id(props.id.clone())
            .bg(bg)
            .border_1()
            .border_color(border)
            .rounded(radius)
            .p(pad)
            .shadow(vec![gpui::BoxShadow {
                color: gpui::hsla(0.0, 0.0, 0.0, alpha),
                blur_radius: gpui::px(12.0),
                spread_radius: gpui::px(0.0),
                offset: gpui::Point { x: gpui::px(0.0), y: gpui::px(4.0) },
                
            }])
    }
}

pub fn arc_menu<T: MenuRenderer + 'static>(r: T) -> Arc<dyn MenuRenderer> {
    Arc::new(r)
}
