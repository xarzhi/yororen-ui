//! Global state for the theme-compare demo.

use gpui::{App, AppContext, Entity, Global};

/// Which side of the window is currently using the mini theme. The
/// left half always shows the system palette; the right half can be
/// toggled between system and mini.
pub struct ThemeCompareState {
    pub right_uses_mini: Entity<bool>,
}

impl Global for ThemeCompareState {}

impl ThemeCompareState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            right_uses_mini: cx.new(|_| true),
        }
    }

    pub fn right_uses_mini(&self, cx: &App) -> bool {
        *self.right_uses_mini.read(cx)
    }
}
