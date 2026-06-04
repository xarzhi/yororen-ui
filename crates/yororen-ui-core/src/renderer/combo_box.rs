//! `ComboBoxRenderer` — visual side of `ComboBox`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ComboBoxRenderState {
    pub open: bool,
    pub disabled: bool,
    pub has_value: bool,
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
    fn bg(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn border(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.has_value {
            theme.content.primary
        } else {
            theme.content.tertiary
        }
    }
    fn search_bg(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn min_height(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn padding(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(theme.tokens.spacing.inset_sm, theme.tokens.spacing.inset_xs)
    }
    fn border_radius(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
}

pub fn arc_combo_box<T: ComboBoxRenderer + 'static>(r: T) -> Arc<dyn ComboBoxRenderer> {
    Arc::new(r)
}
