//! `SliderRenderer` — visual contract for `Slider`.
//!
//! Unlike most renderers, `SliderRenderer::compose` returns a
//! [`SliderRenderOutput`] that carries both the visual `Div` and a
//! shared `track_bounds` store. The headless layer needs the bounds
//! to convert window-relative mouse events into local track
//! coordinates for drag handling.

use std::any::Any;
use std::sync::{Arc, Mutex};

use gpui::{App, Bounds, Pixels, Stateful};

use crate::headless::slider::SliderProps;

/// Projection of `SliderProps` used by built-in renderers when
/// they want to factor out helpers. Not part of the
/// `SliderRenderer` trait surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct SliderRenderState {
    pub disabled: bool,
}

/// The result of `SliderRenderer::compose`. In addition to the
/// visual tree, the renderer must provide a way for the headless
/// layer to read the painted track bounds.
pub struct SliderRenderOutput {
    pub visual: Stateful<gpui::Div>,
    pub track_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
}

pub trait SliderRenderer: Any + Send + Sync {
    /// Build the full visual tree for a slider and return the
    /// shared bounds store so the headless layer can attach drag
    /// handlers.
    fn compose(&self, props: &SliderProps, cx: &App) -> SliderRenderOutput;
}
