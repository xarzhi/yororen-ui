//! `DropdownMenuRenderer` — visual contract for `DropdownMenu`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (trigger_bg / trigger_hover_bg / trigger_fg /
//! min_height / border_radius / chevron_rotation)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::dropdown_menu::DropdownMenuProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct DropdownMenuRenderState {
    pub open: bool,
}

pub trait DropdownMenuRenderer: Any + Send + Sync {
    fn compose(&self, props: &DropdownMenuProps, cx: &App) -> Div;
}
