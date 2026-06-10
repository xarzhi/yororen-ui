//! `LabelRenderer` — visual contract for `Label`.
//!
//! Trait surface is just `compose`. The renderer takes the
//! full `LabelProps` and returns a styled `Div`. Labels are
//! a non-interactive primitive: no focus, no callbacks, no
//! `Stateful<_>` wrapper required.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::label::LabelProps;

/// Projection of `LabelProps` used by built-in renderers when
/// they want to factor out helpers. Not part of the
/// `LabelRenderer` trait surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct LabelRenderState {
    pub muted: bool,
    pub strong: bool,
    pub mono: bool,
    pub inherit_color: bool,
    pub ellipsis: bool,
    pub wrap: bool,
    pub max_lines: Option<usize>,
}

pub trait LabelRenderer: Any + Send + Sync {
    /// Build the styled `Div` for a label. The renderer is
    /// responsible for colour, weight, font family, ellipsis
    /// and max_lines behaviour.
    fn compose(&self, props: &LabelProps, cx: &App) -> Div;
}
