//! `ListboxRenderer` — visual contract for `Listbox`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / border / hover_bg / selected_bg / fg / padding /
//! min_height / border_radius) stay on the concrete renderer
//! type. The headless layer owns the option list, highlight
//! index, and selected value; the renderer decides how those
//! are painted into a `Stateful<Div>` containing one row per
//! option.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::listbox::ListboxProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ListboxRenderState {
    pub row_count: usize,
}

pub trait ListboxRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the listbox. The
    /// renderer iterates `props.state.options` and emits one
    /// row per option, applying the highlight / selected /
    /// hover styling of its choice. Click handlers are wired
    /// to `state.pick(value, …)`; the headless layer fires
    /// `on_change` on the user's behalf.
    fn compose(&self, props: &ListboxProps, cx: &App) -> Stateful<Div>;
}