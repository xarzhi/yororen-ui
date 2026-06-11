//! `RadioGroupRenderer` — visual contract for `RadioGroup`.
//!
//! Trait surface is just `compose`. The renderer may apply a default
//! layout (row / column gap) and focus ring styling; the headless
//! layer owns the selected index and change callbacks.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::radio_group::RadioGroupProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct RadioGroupRenderState {
    pub selected_index: Option<usize>,
}

pub trait RadioGroupRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the radio group. Radio
    /// buttons are added by the caller as children after `.render(cx)`.
    fn compose(&self, props: &RadioGroupProps, cx: &App) -> Stateful<Div>;
}
