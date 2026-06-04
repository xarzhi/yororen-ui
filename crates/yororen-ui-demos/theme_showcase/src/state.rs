//! Global state for the theme-showcase demo.

use gpui::{App, AppContext, Entity, Global};

/// Which theme the demo is currently using. The per-element
/// `with_theme` override was removed, so the demo flips the global
/// theme instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RightThemeKind {
    /// v0.5 system + token renderers.
    #[default]
    System,
    /// Catppuccin Mocha palette + Catppuccin renderers.
    Catppuccin,
    /// Material Design 3 palette + Material renderers.
    Material,
    /// v0.5 system palette but with the Catppuccin renderers
    /// layered on top. Demonstrates that renderer swap and palette
    /// swap are independent.
    CatppuccinRenderersOnSystemPalette,
}

pub struct ThemeShowcaseState {
    pub kind: Entity<RightThemeKind>,
}

impl Global for ThemeShowcaseState {}

impl ThemeShowcaseState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            kind: cx.new(|_| RightThemeKind::default()),
        }
    }
}
