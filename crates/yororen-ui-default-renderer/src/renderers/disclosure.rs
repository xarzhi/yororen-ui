//! `TokenDisclosureRenderer` — default `DisclosureRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::disclosure::DisclosureProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::disclosure::{DisclosureRenderState, DisclosureRenderer};

pub struct TokenDisclosureRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenDisclosureRenderer {
    pub fn trigger_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn trigger_fg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn trigger_hover_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    pub fn min_height(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn chevron_rotation(&self, state: &DisclosureRenderState, _theme: &Theme) -> f32 {
        if state.open { 90.0 } else { 0.0 }
    }
    pub fn body_padding(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32)
    }
}

impl DisclosureRenderer for TokenDisclosureRenderer {
    fn compose(&self, props: &DisclosureProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DisclosureRenderState { open: props.open };
        let bg = self.trigger_bg(&state, theme);
        let fg = self.trigger_fg(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let rot = self.chevron_rotation(&state, theme);
        let chev_str = if props.open { "▼" } else { "▶" };
        let _ = rot; // rotation only meaningful in a Stateful context
        div()
            .flex()
            .items_center()
            .gap(px_from(8.0))
            .bg(bg)
            .text_color(fg)
            .min_h(h)
            .rounded(r)
            .px(px_from(12.0))
            .child(chev_str)
            .child(props.title.clone())
    }
}

fn px_from(v: f32) -> gpui::Pixels {
    gpui::px(v)
}

pub fn arc_disclosure<T: DisclosureRenderer + 'static>(r: T) -> Arc<dyn DisclosureRenderer> {
    Arc::new(r)
}
