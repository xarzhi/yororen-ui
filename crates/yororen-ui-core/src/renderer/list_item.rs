//! `ListItemRenderer` — visual contract for `ListItem`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / hover_bg / selected_bg / fg / padding /
//! min_height / border_radius) stay on the concrete
//! renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::list_item::ListItemProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ListItemRenderState {
    pub selected: bool,
    pub disabled: bool,
    pub hovered: bool,
}

pub trait ListItemRenderer: Any + Send + Sync {
    fn compose(&self, props: &ListItemProps, cx: &App) -> Div;
}
