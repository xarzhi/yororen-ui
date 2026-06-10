//! `TokenComboBoxRenderer` — default `ComboBoxRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::combo_box::ComboBoxProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::combo_box::{ComboBoxRenderState, ComboBoxRenderer};

pub struct TokenComboBoxRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenComboBoxRenderer {
    pub fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    pub fn border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    pub fn focus_border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    pub fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.custom_fg.is_some() {
            state.custom_fg.unwrap()
        } else if state.has_value {
            theme.get_color("content.primary").unwrap_or_default()
        } else {
            theme.get_color("content.tertiary").unwrap_or_default()
        }
    }
    pub fn search_bg(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn padding(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    pub fn border_radius(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

impl ComboBoxRenderer for TokenComboBoxRenderer {
    fn compose(&self, props: &ComboBoxProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state_read = props.state.read(cx);
        let state = ComboBoxRenderState {
            open: state_read.is_open(),
            disabled: false,
            has_value: state_read.value.is_some(),
            custom_bg: None,
            custom_border: None,
            custom_focus_border: None,
            custom_fg: None,
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        div()
            .flex()
            .items_center()
            .bg(bg)
            .border_color(border)
            .text_color(fg)
            .p(pad.top)
            .min_h(h)
            .rounded(r)
            .child(state_read.placeholder.to_string())
    }
}

pub fn arc_combo_box<T: ComboBoxRenderer + 'static>(r: T) -> Arc<dyn ComboBoxRenderer> {
    Arc::new(r)
}
