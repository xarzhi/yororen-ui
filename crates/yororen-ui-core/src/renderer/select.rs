//! `SelectRenderer` — visual contract for `Select`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / border / focus_border / fg / hint_color /
//! min_height / padding / border_radius / chevron_rotation)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::select::SelectProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct SelectRenderState {
    pub open: bool,
    pub disabled: bool,
    pub has_value: bool,
    pub custom_bg: Option<gpui::Hsla>,
    pub custom_border: Option<gpui::Hsla>,
    pub custom_focus_border: Option<gpui::Hsla>,
    pub custom_fg: Option<gpui::Hsla>,
}

pub trait SelectRenderer: Any + Send + Sync {
    fn compose(&self, props: &SelectProps, cx: &App) -> Div;
}
