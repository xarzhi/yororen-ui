//! `AvatarRenderer` — the visual side of `Avatar`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct AvatarRenderState {
    pub has_custom_bg: bool,
    pub has_status: bool,
    pub is_circle: bool,
}

pub trait AvatarRenderer: Any + Send + Sync {
    fn default_bg(&self, state: &AvatarRenderState, theme: &Theme) -> Hsla;
    fn border_radius(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels;
    fn status_dot_size(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels;
    fn status_inset(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels;
    fn status_border_w(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels;
    fn status_border_color(&self, state: &AvatarRenderState, theme: &Theme) -> Hsla;
}

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
