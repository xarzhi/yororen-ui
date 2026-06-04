//! Global state for the theme-showcase demo.

use gpui::{App, AppContext, Entity, Global};

/// Which theme the right half of the window uses. The left half
/// always shows the system palette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RightThemeKind {
    /// Same as left half: v0.5 system + token renderers.
    System,
    /// Catppuccin Mocha palette + Catppuccin renderers.
    #[default]
    Catppuccin,
    /// Material Design 3 palette + Material renderers (Phase H.1,
    /// the second official theme).
    Material,
    /// v0.5 system palette but with the Catppuccin renderers
    /// layered on top. Demonstrates that renderer swap and palette
    /// swap are independent.
    CatppuccinRenderersOnSystemPalette,
}

pub struct ThemeShowcaseState {
    pub right_kind: Entity<RightThemeKind>,
}

impl Global for ThemeShowcaseState {}

impl ThemeShowcaseState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            right_kind: cx.new(|_| RightThemeKind::default()),
        }
    }

    pub fn right_kind(&self, cx: &gpui::App) -> RightThemeKind {
        *self.right_kind.read(cx)
    }
}
