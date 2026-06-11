//! `OverlayRenderer` — visual contract for `Overlay`.
//!
//! Trait surface is just `compose`. The renderer paints the scrim;
//! the headless layer owns open state, focus trapping, and
//! dismissal callbacks.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::overlay::OverlayProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct OverlayRenderState {
    pub open: bool,
}

pub trait OverlayRenderer: Any + Send + Sync {
    /// Build the scrim `Stateful<Div>`. The headless `render(cx)`
    /// will layer interaction (focus trap / Esc / scrim click) on
    /// top of this element.
    fn compose(&self, props: &OverlayProps, cx: &App) -> Stateful<Div>;
}
