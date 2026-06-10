//! `TokenTooltipRenderer` — default `TooltipRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::tooltip::TooltipProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::tooltip::{TooltipRenderState, TooltipRenderer};

pub struct TokenTooltipRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenTooltipRenderer {
    pub fn bg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn fg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn padding(&self, _state: &TooltipRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
        )
    }
    pub fn font_size(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_sm")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
}

impl TooltipRenderer for TokenTooltipRenderer {
    fn compose(&self, props: &TooltipProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TooltipRenderState {
            has_custom_bg: props.has_custom_bg,
            has_custom_fg: props.has_custom_fg,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let fs = self.font_size(&state, theme);
        let r = self.border_radius(&state, theme);
        let open = props.state.read(cx).is_open();
        let mut el = div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .p(pad.top)
            .text_size(fs)
            .rounded(r)
            .child(props.text.clone());
        if !open {
            el = el.invisible();
        }
        el
    }
}

pub fn arc_tooltip<T: TooltipRenderer + 'static>(r: T) -> Arc<dyn TooltipRenderer> {
    Arc::new(r)
}
