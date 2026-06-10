//! `TokenCardRenderer` — default `CardRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, Pixels, Styled, div};

use yororen_ui_core::headless::card::CardProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::card::{CardRenderState, CardRenderer};

pub struct TokenCardRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenCardRenderer {
    pub fn bg(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    pub fn padding(&self, _state: &CardRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32,
        ))
    }
    pub fn border_radius(&self, _state: &CardRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(0.0) as f32)
    }
    pub fn shadow_alpha(&self, _state: &CardRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_1").unwrap_or_default().a
    }
}

impl CardRenderer for TokenCardRenderer {
    fn compose(&self, props: &CardProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = CardRenderState {
            has_custom_bg: props.has_custom_bg,
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let pad = self.padding(&state, theme);
        let r = self.border_radius(&state, theme);
        div().bg(bg).border_color(border).p(pad.top).rounded(r)
    }
}

pub fn arc_card<T: CardRenderer + 'static>(r: T) -> Arc<dyn CardRenderer> {
    Arc::new(r)
}
