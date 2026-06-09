//! `ComboBoxRenderer` — visual side of `ComboBox`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ComboBoxRenderState {
    pub open: bool,
    pub disabled: bool,
    pub has_value: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait ComboBoxRenderer: Any + Send + Sync {
    fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn search_bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &ComboBoxRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &ComboBoxRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenComboBoxRenderer;

impl ComboBoxRenderer for TokenComboBoxRenderer {
    fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    fn border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    fn focus_border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
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
    fn search_bg(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn min_height(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    fn padding(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_combo_box<T: ComboBoxRenderer + 'static>(r: T) -> Arc<dyn ComboBoxRenderer> {
    Arc::new(r)
}
