//! `TreeItemRenderer` — visual contract for `TreeItem`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / hover_bg / selected_bg / fg / indent / padding /
//! min_height / chevron_size) stay on the concrete
//! renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::tree_item::TreeItemProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct TreeItemRenderState {
    pub selected: bool,
    pub expanded: bool,
    pub depth: u8,
    pub is_leaf: bool,
}

pub trait TreeItemRenderer: Any + Send + Sync {
    fn compose(&self, props: &TreeItemProps, cx: &App) -> Div;
}
