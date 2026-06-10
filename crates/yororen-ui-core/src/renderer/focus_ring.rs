//! `FocusRingRenderer` — visual contract for `FocusRing`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (color / width) stay on the concrete renderer type.
//!
//! `compose` returns the *ring overlay* `Stateful<Div>` —
//! a wrapper with border color/width set, ready to receive
//! a child. The bound `FocusHandle` is bound via
//! `track_focus`, so the ring is visible only while that
//! handle is the active keyboard focus.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::focus_ring::FocusRingProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct FocusRingRenderState {
    pub has_custom_color: bool,
}

pub trait FocusRingRenderer: Any + Send + Sync {
    fn compose(&self, props: &FocusRingProps, cx: &App) -> Stateful<Div>;
}
