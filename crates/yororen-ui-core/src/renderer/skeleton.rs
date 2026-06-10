//! `SkeletonRenderer` — visual contract for `Skeleton`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / min_height / border_radius) stay on the concrete
//! renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::skeleton::SkeletonProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct SkeletonRenderState {
    pub block: bool,
    pub block_sharp: bool,
    pub rounded: bool,
}

pub trait SkeletonRenderer: Any + Send + Sync {
    fn compose(&self, props: &SkeletonProps, cx: &App) -> Div;
}
