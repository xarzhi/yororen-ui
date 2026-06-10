//! `EmptyStateRenderer` — visual contract for `EmptyState`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (icon_color / title_color / body_color / padding / icon_size /
//! gap) stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::empty_state::EmptyStateProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct EmptyStateRenderState {}

pub trait EmptyStateRenderer: Any + Send + Sync {
    fn compose(&self, props: &EmptyStateProps, cx: &App) -> Div;
}
