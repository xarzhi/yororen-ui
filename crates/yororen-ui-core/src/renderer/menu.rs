//! `MenuRenderer` — visual contract for `Menu`.
//!
//! Trait surface is just `compose`. The renderer paints the menu
//! shell (background, border, shadow, padding). The headless layer
//! owns item highlight / selection state.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::menu::MenuProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct MenuRenderState {}

pub trait MenuRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the menu shell. Children
    /// (menu items) are added by the caller after `.render(cx)`.
    fn compose(&self, props: &MenuProps, cx: &App) -> Stateful<Div>;
}
