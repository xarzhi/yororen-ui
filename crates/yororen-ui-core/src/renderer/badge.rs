//! `BadgeRenderer` — visual contract for `Badge`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / fg / padding_x / height / font_size / font_weight /
//! border_radius) stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::badge::BadgeProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct BadgeRenderState {
    pub variant: crate::headless::badge::BadgeVariant,
    pub has_custom_tone: bool,
}

pub trait BadgeRenderer: Any + Send + Sync {
    fn compose(&self, props: &BadgeProps, cx: &App) -> Div;
}
