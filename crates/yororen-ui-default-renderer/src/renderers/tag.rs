//! `TokenTagRenderer` — default `TagRenderer` impl.

use std::sync::Arc;

use gpui::{
    App, Div, FontWeight, Hsla, ParentElement, Pixels, Styled, div,
};

use yororen_ui_core::headless::tag::TagProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::tag::{TagRenderState, TagRenderer};

pub struct TokenTagRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenTagRenderer {
    pub fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.get_color("action.primary.bg").unwrap_or_default()
        } else if state.has_custom_tone {
            theme.get_color("action.neutral.bg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.bg").unwrap_or_default()
        }
    }

    pub fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else if state.has_custom_tone {
            theme.get_color("content.on_status").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.fg").unwrap_or_default()
        }
    }

    pub fn min_height(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.tag.min_height")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn padding_x(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32)
    }

    pub fn font_size(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_xs")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn font_weight(&self, _state: &TagRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_medium")
                .unwrap_or(500.0) as f32,
        )
    }

    pub fn border_radius(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
    }

    pub fn close_size(&self, _state: &TagRenderState, _theme: &Theme) -> Pixels {
        gpui::px(16.)
    }

    pub fn close_hover_bg(&self, _state: &TagRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
}

impl TagRenderer for TokenTagRenderer {
    fn compose(&self, props: &TagProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TagRenderState {
            selected: props.selected,
            has_custom_tone: false,
            closable: props.closable,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let h = self.min_height(&state, theme);
        let p = self.padding_x(&state, theme);
        let fs = self.font_size(&state, theme);
        let fw = self.font_weight(&state, theme);
        let r = self.border_radius(&state, theme);
        let mut el = div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .min_h(h)
            .px(p)
            .text_size(fs)
            .font_weight(fw)
            .rounded(r)
            .gap(p / 2.)
            .child(props.label.clone());
        if props.closable {
            let close_size = self.close_size(&state, theme);
            el = el.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(close_size)
                    .rounded(close_size / 2.)
                    .child("×"),
            );
        }
        el
    }
}

pub fn arc_tag<T: TagRenderer + 'static>(r: T) -> Arc<dyn TagRenderer> {
    Arc::new(r)
}
