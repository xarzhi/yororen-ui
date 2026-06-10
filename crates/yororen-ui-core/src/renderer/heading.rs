//! `HeadingRenderer` — visual contract for `Heading`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (size / weight / color) stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::heading::HeadingProps;

#[derive(Clone, Copy, Debug)]
pub struct HeadingRenderState {
    pub level: crate::headless::heading::HeadingLevel,
}

pub trait HeadingRenderer: Any + Send + Sync {
    fn compose(&self, props: &HeadingProps, cx: &App) -> Div;
}
