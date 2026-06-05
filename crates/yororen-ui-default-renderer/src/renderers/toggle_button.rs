//! `ToggleButtonRenderer` — visual side of `ToggleButton`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::button::ActionVariantKind;
use yororen_ui_core::theme::Theme;

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

pub trait ToggleButtonRenderer: Any + Send + Sync {
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
                return theme.get_color("action.primary.bg").unwrap_or_default();
            }
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.disabled {
            theme.get_color("action.neutral.disabled_bg").unwrap_or_default()
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
    fn min_height(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.toggle_button.min_height").unwrap_or(0.0) as f32)
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

use gpui::{div, App, Stateful, Styled};
use yororen_ui_core::headless::toggle_button::ToggleButtonProps;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultToggleButton: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultToggleButton for ToggleButtonProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn ToggleButtonRenderer> = cx
            .renderer_arc::<markers::ToggleButton, dyn ToggleButtonRenderer>()
            .expect("ToggleButtonRenderer registered");
        let state = ToggleButtonRenderState::default();
        let bg = r.bg(&state, theme);
        let fg = r.fg(&state, theme);
        let min_h = r.min_height(&state, theme);
        let radius = r.border_radius(&state, theme);
        let el = div()
            .bg(bg)
            .text_color(fg)
            .min_h(min_h)
            .rounded(radius)
            .px(gpui::px(12.))
            .py(gpui::px(6.))
            .flex()
            .items_center()
            .justify_center();
        self.apply(el)
    }
}
