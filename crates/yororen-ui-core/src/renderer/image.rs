//! `ImageRenderer` — visual contract for `Image`.
//!
//! Trait surface is just `compose`. Inherent helpers (placeholder
//! bg / default fit / etc.) stay on the concrete renderer type.

use std::any::Any;

use gpui::{Div, Stateful};

use crate::headless::image::ImageProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ImageRenderState {}

pub trait ImageRenderer: Any + Send + Sync {
    fn compose(&self, props: &ImageProps, cx: &gpui::App) -> Stateful<Div>;
}
