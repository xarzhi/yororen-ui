//! `AvatarRenderer` — the visual side of `Avatar`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

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
