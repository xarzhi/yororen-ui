//! `ProgressBarRenderer` — visual contract for `ProgressBar`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (track / fill / height / border_color / border_radius)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::progress::ProgressBarProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProgressBarRenderState {
    pub indeterminate: bool,
    pub has_custom_height: bool,
}

pub trait ProgressBarRenderer: Any + Send + Sync {
    fn compose(&self, props: &ProgressBarProps, cx: &App) -> Div;
}
