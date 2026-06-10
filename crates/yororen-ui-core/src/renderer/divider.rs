//! `DividerRenderer` — visual contract for `Divider`.
//!
//! Trait surface is just `compose`. The renderer returns a
//! plain `Div` with the chosen bg + thickness. The caller can
//! still chain `.w_full()` / `.h_full()` on the returned `Div`
//! to control the long dimension.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::divider::DividerProps;

/// Projection of `DividerProps` used by built-in renderers when
/// they want to factor out helpers. Not part of the
/// `DividerRenderer` trait surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct DividerRenderState {
    pub horizontal: bool,
}

pub trait DividerRenderer: Any + Send + Sync {
    /// Build the styled `Div` for a divider.
    fn compose(&self, props: &DividerProps, cx: &App) -> Div;
}
