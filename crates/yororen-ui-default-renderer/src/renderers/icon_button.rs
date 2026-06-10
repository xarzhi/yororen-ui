//! `IconButtonRenderer` — visual side of `IconButton`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::renderer::variant::ActionVariantKind;
use yororen_ui_core::theme::Theme;

use yororen_ui_core::renderer::variant::{VariantState, VariantStyle};

pub use yororen_ui_core::renderer::icon_button::{IconButtonRenderState, IconButtonRenderer};

pub struct TokenIconButtonRenderer;

fn action_variant_key(variant: ActionVariantKind) -> &'static str {
    match variant {
        ActionVariantKind::Neutral => "neutral",
        ActionVariantKind::Primary => "primary",
        ActionVariantKind::Danger => "danger",
    }
}

impl IconButtonRenderer for TokenIconButtonRenderer {
    fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let key = action_variant_key(state.variant);
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        theme
            .get_color(&format!("action.{}.{}", key, field))
            .unwrap_or_default()
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
        let key = action_variant_key(state.variant);
        theme
            .get_color(&format!("action.{}.hover_bg", key))
            .unwrap_or_default()
    }
    fn active_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let key = action_variant_key(state.variant);
        theme
            .get_color(&format!("action.{}.active_bg", key))
            .unwrap_or_default()
    }
    fn size(&self, _state: &IconButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.icon_button_min_size")
                .unwrap_or(0.0) as f32,
        )
    }
    fn border_radius(&self, _state: &IconButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
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

// =====================================================================
// `DefaultIconButton` — `headless::IconButtonProps` sugar.
// =====================================================================

use gpui::{App, InteractiveElement, Stateful, StatefulInteractiveElement, Styled, div};
use yororen_ui_core::headless::icon_button::IconButtonProps;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::ActiveTheme;
