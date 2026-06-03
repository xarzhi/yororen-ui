//! `IconButtonRenderer` — visual side of `IconButton`.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::{ActionVariantKind, Theme};

use super::variant::{VariantState, VariantStyle};

#[derive(Clone, Debug, Default)]
pub struct IconButtonRenderState {
    pub variant: ActionVariantKind,
    pub disabled: bool,
    pub has_custom_bg: bool,
    pub has_custom_hover_bg: bool,
    /// Pre-resolved custom variant from the global `VariantRegistry`.
    /// When `Some`, the renderer delegates color decisions to it.
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

pub trait IconButtonRenderer: Send + Sync {
    fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla;
    fn hover_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla;
    fn size(&self, state: &IconButtonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &IconButtonRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &IconButtonRenderState, theme: &Theme) -> f32;
}

pub struct TokenIconButtonRenderer;

impl IconButtonRenderer for TokenIconButtonRenderer {
    fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let v = theme.action_variant(state.variant);
        if state.disabled { v.disabled_bg } else { v.bg }
    }
    fn hover_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            // For hover, the registered variant does not get a hover
            // signal, but its base bg works as a sensible default. The
            // user can still override via `.hover_bg(...)` on the
            // builder.
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        theme.action_variant(state.variant).hover_bg
    }
    fn size(&self, _state: &IconButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.icon_button_min_size
    }
    fn border_radius(&self, _state: &IconButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn disabled_opacity(&self, state: &IconButtonRenderState, _theme: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        1.0
    }
}

pub fn arc_icon_button<T: IconButtonRenderer + 'static>(r: T) -> Arc<dyn IconButtonRenderer> {
    Arc::new(r)
}
