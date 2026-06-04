//! Global state for the flavor gallery demo.

use gpui::{App, AppContext, Entity, Global};

/// The 6 flavors the demo can display. "System" uses the system
/// theme (default light/dark from the OS appearance); the 4 in
/// the middle are the Catppuccin flavors; "Material 3" is the
/// second official theme package.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlavorKind {
    /// Use the system palette. Light/dark follows the OS appearance.
    #[default]
    System,
    /// Catppuccin Latte (light).
    Latte,
    /// Catppuccin Frappé (medium-dark).
    Frappe,
    /// Catppuccin Macchiato (darker than Frappé).
    Macchiato,
    /// Catppuccin Mocha (darkest, most popular).
    Mocha,
    /// Material Design 3 (the second official theme).
    Material,
}

impl FlavorKind {
    /// Canonical lowercase name used in the UI labels.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Latte => "Latte",
            Self::Frappe => "Frappé",
            Self::Macchiato => "Macchiato",
            Self::Mocha => "Mocha",
            Self::Material => "Material 3",
        }
    }

    /// All 6 variants in canonical order.
    pub const ALL: [FlavorKind; 6] = [
        Self::System,
        Self::Latte,
        Self::Frappe,
        Self::Macchiato,
        Self::Mocha,
        Self::Material,
    ];
}

impl std::fmt::Display for FlavorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Global state for the demo. Holds the active flavor and the
/// visibility flags for the per-flavor modal that the user opens
/// via the "Show modal" button.
pub struct FlavorGalleryState {
    /// The 4 (now 5) columns always show a modal, but only one
    /// is open at a time. We track an enum to know which one.
    pub active_modal: Entity<ActiveModal>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActiveModal {
    #[default]
    None,
    /// The user clicked "Show modal" in one of the columns. The
    /// variant identifies which column.
    Column(FlavorKind),
}

impl Global for FlavorGalleryState {}

impl FlavorGalleryState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            active_modal: cx.new(|_| ActiveModal::default()),
        }
    }
}

// Suppress an unused warning on _kind_passthrough when the
// demo is built with all features.
#[allow(dead_code)]
fn _suppress_unused_warning() {
    let _ = FlavorKind::ALL;
}
