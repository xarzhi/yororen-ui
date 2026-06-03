//! `ToggleButtonRenderer` — visual side of `ToggleButton`.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::{ActionVariantKind, Theme};

use super::variant::{VariantState, VariantStyle};

#[derive(Clone, Debug, Default)]
pub struct ToggleButtonRenderState {
    pub variant: ActionVariantKind,
    pub selected: bool,
    pub disabled: bool,
    /// Pre-resolved custom variant from the global `VariantRegistry`.
    /// When `Some`, the renderer uses it for the *unselected* colors
    /// (the selected appearance still maps to the action.primary slot
    /// so toggle semantics stay intact). Disabled colors are
    /// automatically handled by the variant's own `disabled` branch.
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

pub trait ToggleButtonRenderer: Send + Sync {
    fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &ToggleButtonRenderState, theme: &Theme) -> f32;
}

pub struct TokenToggleButtonRenderer;

impl ToggleButtonRenderer for TokenToggleButtonRenderer {
    fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            // ToggleButton has a binary visual state (selected vs. not);
            // the registered custom variant controls the unselected
            // look. When selected we keep mapping to theme.primary so
            // existing toggle semantics are preserved.
            if state.selected {
                return theme.action.primary.bg;
            }
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.disabled {
            theme.action.neutral.disabled_bg
        } else if state.selected {
            theme.action.primary.bg
        } else {
            theme.action.neutral.bg
        }
    }
    fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            if state.selected {
                return theme.action.primary.fg;
            }
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.selected {
            theme.action.primary.fg
        } else {
            theme.action.neutral.fg
        }
    }
    fn min_height(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.toggle_button.min_height
    }
    fn border_radius(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn disabled_opacity(&self, state: &ToggleButtonRenderState, _theme: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        1.0
    }
}

pub fn arc_toggle_button<T: ToggleButtonRenderer + 'static>(r: T) -> Arc<dyn ToggleButtonRenderer> {
    Arc::new(r)
}
