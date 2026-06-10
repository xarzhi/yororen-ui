//! `PanelRenderer` — visual contract for `Panel`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / border / padding / border_radius / shadow_alpha)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::panel::PanelProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct PanelRenderState {
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_padding: bool,
}

pub trait PanelRenderer: Any + Send + Sync {
    fn compose(&self, props: &PanelProps, cx: &App) -> Div;
}
