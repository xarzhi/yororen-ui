//! `ButtonGroupRenderer` — visual contract for `ButtonGroup`.
//!
//! Unlike most renderers (which take `&XxxProps` and return a
//! bare `Div`), `ButtonGroupRenderer` **takes ownership** of
//! the props and returns a `Stateful<Div>`. This is because
//! the renderer needs to consume the styled children the
//! caller stored via `ButtonGroupProps::child(...)` and apply
//! the segmented-control corner rounding to them — `Stateful<Div>`
//! is not `Clone`, so the children can only be moved out of
//! the props once.
//!
//! The headless `ButtonGroupProps::render(cx)` method is a
//! thin pass-through: `r.compose(self, cx)`.

use std::any::Any;

use gpui::{App, Stateful};

use crate::headless::button_group::ButtonGroupOrientation;
use crate::headless::button_group::ButtonGroupProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonGroupRenderState {
    pub orientation: ButtonGroupOrientation,
    pub attached: bool,
}

pub trait ButtonGroupRenderer: Any + Send + Sync {
    fn compose(&self, props: ButtonGroupProps, cx: &App) -> Stateful<gpui::Div>;
}
