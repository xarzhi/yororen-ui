//! `FormFieldRenderer` — visual contract for `FormField`.
//!
//! Trait surface is just `compose`. The renderer lays out the
//! label, required indicator, help text, error text, and the
//! caller-supplied input child.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::form_field::FormFieldProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct FormFieldRenderState {
    pub has_error: bool,
    pub required: bool,
}

pub trait FormFieldRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the form field. The input
    /// element is added by the caller as a child after `.render(cx)`.
    fn compose(&self, props: &FormFieldProps, cx: &App) -> Stateful<Div>;
}
