//! `TokenModalRenderer` — default `ModalRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::modal::ModalProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::modal::{ModalRenderState, ModalRenderer};

pub struct TokenModalRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenModalRenderer {
    pub fn scrim(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        let mut c = theme.get_color("shadow.elevation_2").unwrap_or_default();
        c.a = 0.5;
        c
    }
    pub fn panel_bg(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or_default()
    }
    pub fn panel_border(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    pub fn panel_padding(&self, _state: &ModalRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.spacing.inset_lg").unwrap_or(0.0) as f32,
        ))
    }
    pub fn panel_border_radius(&self, _state: &ModalRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(0.0) as f32)
    }
    pub fn panel_shadow_alpha(&self, _state: &ModalRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_2").unwrap_or_default().a
    }
}

impl ModalRenderer for TokenModalRenderer {
    fn compose(&self, props: &ModalProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ModalRenderState {};
        let _ = props; // state machine is consulted for visibility in the renderer wrapper
        let panel_bg = self.panel_bg(&state, theme);
        let panel_border = self.panel_border(&state, theme);
        let pad = self.panel_padding(&state, theme);
        let r = self.panel_border_radius(&state, theme);
        div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .bg(self.scrim(&state, theme))
            .child(
                div()
                    .bg(panel_bg)
                    .border_color(panel_border)
                    .p(pad.top)
                    .rounded(r),
            )
    }
}

pub fn arc_modal<T: ModalRenderer + 'static>(r: T) -> Arc<dyn ModalRenderer> {
    Arc::new(r)
}
