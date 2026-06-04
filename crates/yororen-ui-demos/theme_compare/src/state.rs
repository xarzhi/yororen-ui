//! Global state for the theme-compare demo.

use gpui::{App, AppContext, Entity, Global};

/// Whether the demo is currently showing the mini theme. The
/// `with_theme` per-element override was removed, so the demo
/// flips the global theme instead.
pub struct ThemeCompareState {
    pub uses_mini: Entity<bool>,
}

impl Global for ThemeCompareState {}

impl ThemeCompareState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            uses_mini: cx.new(|_| false),
        }
    }
}
