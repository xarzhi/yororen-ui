//! `ModalRenderer` — visual contract for `Modal`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (scrim / panel_bg / panel_border / panel_padding /
//! panel_border_radius / panel_shadow_alpha) stay on the
//! concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::modal::ModalProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ModalRenderState {}

pub trait ModalRenderer: Any + Send + Sync {
    fn compose(&self, props: &ModalProps, cx: &App) -> Div;
}
