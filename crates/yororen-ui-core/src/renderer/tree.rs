//! `TreeRenderer` — visual contract for `Tree`.
//!
//! Trait surface is just `compose`. The renderer paints the tree
//! container; individual rows are rendered by `TreeItemRenderer`.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::tree::TreeProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct TreeRenderState {
    pub has_selection: bool,
}

pub trait TreeRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the tree. Tree rows are
    /// added by the caller as children after `.render(cx)`.
    fn compose(&self, props: &TreeProps, cx: &App) -> Stateful<Div>;
}
