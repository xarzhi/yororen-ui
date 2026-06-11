//! `ShortcutHintRenderer` — visual contract for `ShortcutHint`.
//!
//! Trait surface is just `compose`. Inherent helpers (label fg /
//! kbd bg / kbd fg / gap) stay on the concrete renderer type.

use std::any::Any;

use gpui::{Div, Stateful};

use crate::headless::shortcut_hint::ShortcutHintProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ShortcutHintRenderState {}

pub trait ShortcutHintRenderer: Any + Send + Sync {
    fn compose(&self, props: &ShortcutHintProps, cx: &gpui::App) -> Stateful<Div>;
}
