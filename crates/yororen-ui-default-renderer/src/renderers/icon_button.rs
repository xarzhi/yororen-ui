//! `IconButtonRenderer` — visual side of `IconButton`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::button::ActionVariantKind;
use yororen_ui_core::theme::Theme;

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

pub trait IconButtonRenderer: Any + Send + Sync {
    fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla;
    fn hover_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla;
    fn size(&self, state: &IconButtonRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &IconButtonRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &IconButtonRenderState, theme: &Theme) -> f32;
}

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
        theme.get_color(&format!("action.{}.{}", key, field)).unwrap_or_default()
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
        theme.get_color(&format!("action.{}.hover_bg", key)).unwrap_or_default()
    }
    fn size(&self, _state: &IconButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.button.icon_button_min_size").unwrap_or(0.0) as f32)
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

use gpui::{prelude::FluentBuilder, div, App, ParentElement, Stateful, Styled};
use yororen_ui_core::headless::icon_button::IconButtonProps;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultIconButton: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultIconButton for IconButtonProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn IconButtonRenderer> = cx
            .renderer_arc::<markers::IconButton, dyn IconButtonRenderer>()
            .expect("IconButtonRenderer registered");
        let state = IconButtonRenderState {
            variant: Default::default(),
            disabled: self.disabled,
            has_custom_bg: false,
            has_custom_hover_bg: false,
            custom_style: None,
        };
        let bg = r.bg(&state, theme);
        let radius = r.border_radius(&state, theme);
        let opacity = if self.disabled {
            r.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let el = div()
            .bg(bg)
            .rounded(radius)
            .size(gpui::px(36.))
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center();
        self.apply(el)
    }
}
