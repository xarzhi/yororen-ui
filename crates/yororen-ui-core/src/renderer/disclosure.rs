//! `DisclosureRenderer` — visual contract for `Disclosure`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (trigger_bg / trigger_fg / trigger_hover_bg /
//! min_height / border_radius / chevron_rotation /
//! body_padding) stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::disclosure::DisclosureProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct DisclosureRenderState {
    pub open: bool,
}

pub trait DisclosureRenderer: Any + Send + Sync {
    fn compose(&self, props: &DisclosureProps, cx: &App) -> Div;
}
