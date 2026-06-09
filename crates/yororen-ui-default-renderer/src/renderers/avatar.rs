//! `AvatarRenderer` — the visual side of `Avatar`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::avatar::{AvatarRenderState, AvatarRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenAvatarRenderer;

impl AvatarRenderer for TokenAvatarRenderer {
    fn default_bg(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }

    fn border_radius(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels {
        if state.is_circle {
            gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
        } else {
            gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
        }
    }

    fn status_dot_size(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.status_dot_size")
                .unwrap_or(0.0) as f32,
        )
    }

    fn status_inset(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.status_inset")
                .unwrap_or(0.0) as f32,
        )
    }

    fn status_border_w(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.border_w")
                .unwrap_or(0.0) as f32,
        )
    }

    fn status_border_color(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
}

pub fn arc_avatar<T: AvatarRenderer + 'static>(r: T) -> Arc<dyn AvatarRenderer> {
    Arc::new(r)
}
