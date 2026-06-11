//! `SpacerRenderer` — visual contract for `Spacer`.
//!
//! Trait surface is just `compose`. The renderer may apply a
//! default flex grow / shrink so the spacer pushes siblings apart
//! when the caller does not provide explicit sizing.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::spacer::SpacerProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct SpacerRenderState {}

pub trait SpacerRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the spacer.
    fn compose(&self, props: &SpacerProps, cx: &App) -> Stateful<Div>;
}
