//! `TokenIconButtonRenderer` — default `IconButtonRenderer` impl.
//!
//! Trait surface is just `compose`. The helpers below are
//! inherent — they exist so other code can reuse the palette
//! lookups and so unit tests can assert on individual token
//! paths.

use std::sync::Arc;

use gpui::{
    App, Div, ElementId, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div,
};

use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::headless::icon_button::IconButtonProps;
use yororen_ui_core::renderer::variant::ActionVariantKind;
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::icon_button::{IconButtonRenderState, IconButtonRenderer};

pub struct TokenIconButtonRenderer;

fn action_variant_key(variant: ActionVariantKind) -> &'static str {
    match variant {
        ActionVariantKind::Neutral => "neutral",
        ActionVariantKind::Primary => "primary",
        ActionVariantKind::Danger => "danger",
    }
}

// Inherent helpers — *not* part of the `IconButtonRenderer`
// trait surface.
impl TokenIconButtonRenderer {
    pub fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        let key = action_variant_key(state.variant);
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        theme
            .get_color(&format!("action.{}.{}", key, field))
            .unwrap_or_default()
    }
    pub fn fg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        let key = action_variant_key(state.variant);
        let field = if state.disabled { "disabled_fg" } else { "fg" };
        theme
            .get_color(&format!("action.{}.{}", key, field))
            .unwrap_or_default()
    }
    pub fn hover_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        let key = action_variant_key(state.variant);
        theme
            .get_color(&format!("action.{}.hover_bg", key))
            .unwrap_or_default()
    }
    pub fn active_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        let key = action_variant_key(state.variant);
        theme
            .get_color(&format!("action.{}.active_bg", key))
            .unwrap_or_default()
    }
    pub fn size(&self, _state: &IconButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.icon_button_min_size")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &IconButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn disabled_opacity(&self, _state: &IconButtonRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

impl IconButtonRenderer for TokenIconButtonRenderer {
    fn compose(
        &self,
        props: &IconButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = IconButtonRenderState {
            variant: props.variant,
            disabled: props.disabled,
            has_custom_bg: false,
            has_custom_hover_bg: false,
            custom_style: None,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let radius = self.border_radius(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let side = self.size(&state, theme);

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .rounded(radius)
            .size(side)
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_size = props.icon_size;
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(icon_size),
                color: Some(fg),
            }
            .render();
            el = el.child(icon_el);
        }

        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}

pub fn arc_icon_button<T: IconButtonRenderer + 'static>(r: T) -> Arc<dyn IconButtonRenderer> {
    Arc::new(r)
}
