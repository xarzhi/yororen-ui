//! `VirtualListRenderer` — visual contract for `VirtualList`.
//!
//! Trait surface is just `compose`. The current headless API models
//! a virtual list as a scrollable container; the caller supplies
//! visible rows as children after `.render(cx)`.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::virtual_list::VirtualListProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct VirtualListRenderState {
    pub item_count: usize,
}

pub trait VirtualListRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` scrollable container for the
    /// virtual list. Visible items are added by the caller as
    /// children after `.render(cx)`.
    fn compose(&self, props: &VirtualListProps, cx: &App) -> Stateful<Div>;
}
