//! `TokenSplitButtonRenderer` — default `SplitButtonRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::split_button::SplitButtonProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::split_button::{SplitButtonRenderState, SplitButtonRenderer};

pub struct TokenSplitButtonRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenSplitButtonRenderer {
    pub fn primary_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    pub fn primary_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.fg").unwrap_or_default()
    }
    pub fn chevron_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn chevron_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn chevron_hover_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    pub fn min_height(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.split_button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn gap(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.split_button.separator_w")
                .unwrap_or(0.0) as f32,
        )
    }
}

impl SplitButtonRenderer for TokenSplitButtonRenderer {
    fn compose(&self, props: &SplitButtonProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = SplitButtonRenderState {
            open: false,
            disabled: props.disabled,
        };
        let pbg = self.primary_bg(&state, theme);
        let pfg = self.primary_fg(&state, theme);
        let cbg = self.chevron_bg(&state, theme);
        let cfg = self.chevron_fg(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let _ = props; // primary/secondary callbacks wired in headless apply chain
        div()
            .flex()
            .items_center()
            .bg(pbg)
            .text_color(pfg)
            .min_h(h)
            .rounded(r)
            .child(
                div()
                    .flex()
                    .items_center()
                    .px(px_from(12.0))
                    .child("Run"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .bg(cbg)
                    .text_color(cfg)
                    .px(px_from(8.0))
                    .child("▼"),
            )
    }
}

fn px_from(v: f32) -> gpui::Pixels {
    gpui::px(v)
}

pub fn arc_split_button<T: SplitButtonRenderer + 'static>(r: T) -> Arc<dyn SplitButtonRenderer> {
    Arc::new(r)
}
