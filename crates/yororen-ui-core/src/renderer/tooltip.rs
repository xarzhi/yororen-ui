//! `TooltipRenderer` — visual contract for `Tooltip`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / fg / padding / font_size / border_radius) stay
//! on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::tooltip::TooltipProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct TooltipRenderState {
    pub has_custom_bg: bool,
    pub has_custom_fg: bool,
}

pub trait TooltipRenderer: Any + Send + Sync {
    fn compose(&self, props: &TooltipProps, cx: &App) -> Div;
}
