//! `AvatarRenderer` — the visual side of `Avatar`.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct AvatarRenderState {
    pub has_custom_bg: bool,
    pub has_status: bool,
    pub is_circle: bool,
}

pub trait AvatarRenderer: Send + Sync {
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
        theme.surface.hover
    }

    fn border_radius(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels {
        if state.is_circle {
            theme.tokens.radii.pill
        } else {
            theme.tokens.radii.md
        }
    }

    fn status_dot_size(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.avatar.status_dot_size
    }

    fn status_inset(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.avatar.status_inset
    }

    fn status_border_w(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.avatar.border_w
    }

    fn status_border_color(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
}

pub fn arc_avatar<T: AvatarRenderer + 'static>(r: T) -> Arc<dyn AvatarRenderer> {
    Arc::new(r)
}
