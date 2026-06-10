//! `ComboBoxRenderer` — visual contract for `ComboBox`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / border / focus_border / fg / search_bg /
//! min_height / padding / border_radius) stay on the
//! concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::combo_box::ComboBoxProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ComboBoxRenderState {
    pub open: bool,
    pub disabled: bool,
    pub has_value: bool,
    pub custom_bg: Option<gpui::Hsla>,
    pub custom_border: Option<gpui::Hsla>,
    pub custom_focus_border: Option<gpui::Hsla>,
    pub custom_fg: Option<gpui::Hsla>,
}

pub trait ComboBoxRenderer: Any + Send + Sync {
    fn compose(&self, props: &ComboBoxProps, cx: &App) -> Div;
}
