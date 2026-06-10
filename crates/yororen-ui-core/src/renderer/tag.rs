//! `TagRenderer` — visual contract for `Tag`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / fg / min_height / padding_x / font_size /
//! font_weight / border_radius / close_size / close_hover_bg)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::tag::TagProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct TagRenderState {
    pub selected: bool,
    pub has_custom_tone: bool,
    pub closable: bool,
}

pub trait TagRenderer: Any + Send + Sync {
    fn compose(&self, props: &TagProps, cx: &App) -> Div;
}
