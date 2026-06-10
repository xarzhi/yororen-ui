//! `PopoverRenderer` — visual contract for `Popover`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / border / shadow_alpha / border_radius / offset)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::popover::PopoverProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct PopoverRenderState {}

pub trait PopoverRenderer: Any + Send + Sync {
    fn compose(&self, props: &PopoverProps, cx: &App) -> Div;
}
