//! `CardRenderer` — visual contract for `Card`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / border / padding / border_radius / shadow_alpha)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::card::CardProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct CardRenderState {
    pub has_custom_bg: bool,
}

pub trait CardRenderer: Any + Send + Sync {
    fn compose(&self, props: &CardProps, cx: &App) -> Div;
}
