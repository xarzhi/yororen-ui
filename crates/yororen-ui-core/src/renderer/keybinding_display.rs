//! `KeybindingDisplayRenderer` — visual contract for `KeybindingDisplay`.
//!
//! Trait surface is just `compose`. Inherent helpers (kbd bg / kbd
//! fg / radius / padding / gap) stay on the concrete renderer type.

use std::any::Any;

use gpui::{Div, Stateful};

use crate::headless::keybinding_display::KeybindingDisplayProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct KeybindingDisplayRenderState {}

pub trait KeybindingDisplayRenderer: Any + Send + Sync {
    fn compose(&self, props: &KeybindingDisplayProps, cx: &gpui::App) -> Stateful<Div>;
}
