//! `FormRenderer` — visual contract for `Form`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (gap / label_color / error_color / helper_color) stay
//! on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::form::FormProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct FormRenderState {}

pub trait FormRenderer: Any + Send + Sync {
    fn compose(&self, props: &FormProps, cx: &App) -> Div;
}
