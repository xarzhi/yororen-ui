//! `AvatarRenderer` — visual contract for `Avatar`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (default_bg / border_radius / status_dot_size /
//! status_inset / status_border_w / status_border_color)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::avatar::AvatarProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct AvatarRenderState {
    pub has_custom_bg: bool,
    pub has_status: bool,
    pub is_circle: bool,
}

pub trait AvatarRenderer: Any + Send + Sync {
    fn compose(&self, props: &AvatarProps, cx: &App) -> Div;
}
