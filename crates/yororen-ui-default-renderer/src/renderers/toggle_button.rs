//! `ToggleButtonRenderer` — visual side of `ToggleButton`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::renderer::variant::ActionVariantKind;
use yororen_ui_core::theme::Theme;

use yororen_ui_core::renderer::variant::{VariantState, VariantStyle};

pub use yororen_ui_core::renderer::toggle_button::{ToggleButtonRenderState, ToggleButtonRenderer};

pub struct TokenToggleButtonRenderer;

impl ToggleButtonRenderer for TokenToggleButtonRenderer {
    fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            // ToggleButton has a binary visual state (selected vs. not);
            // the registered custom variant controls the unselected
            // look. When selected we keep mapping to theme.primary so
            // existing toggle semantics are preserved.
            if state.selected {
                return theme.get_color("action.primary.bg").unwrap_or_default();
            }
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.disabled {
            theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or_default()
        } else if state.selected {
            theme.get_color("action.primary.bg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.bg").unwrap_or_default()
        }
    }
    fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            if state.selected {
                return theme.get_color("action.primary.fg").unwrap_or_default();
            }
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.fg").unwrap_or_default()
        }
    }
    fn hover_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        // Selected: hover the selected look's hover_bg.
        // Unselected: hover the unselected (neutral) hover_bg.
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or_default();
        }
        if state.selected {
            return theme
                .get_color("action.primary.hover_bg")
                .unwrap_or_default();
        }
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    fn active_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or_default();
        }
        if state.selected {
            return theme
                .get_color("action.primary.active_bg")
                .unwrap_or_default();
        }
        theme
            .get_color("action.neutral.active_bg")
            .unwrap_or_default()
    }
    fn min_height(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.toggle_button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    fn border_radius(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
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

// =====================================================================
// `DefaultToggleButton` — `headless::ToggleButtonProps` sugar.
// =====================================================================

use gpui::{App, InteractiveElement, Stateful, StatefulInteractiveElement, Styled, div};
use yororen_ui_core::headless::toggle_button::ToggleButtonProps;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::ActiveTheme;
