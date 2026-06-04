//! Shared dismiss-behavior enum used by floating components
//! (popover, dropdown menu, combo box, select, modal shell).
//!
//! Replaces the per-component `dismiss_on_escape: bool` flag that
//! all four components used to expose. The variants cover the
//! common configuration surface and let new components share
//! the same API without re-declaring the flag.

/// How a floating component should respond to dismiss events
/// (typically the Escape key, or a scrim click for modals).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DismissBehavior {
    /// Use the component's default. For popover / dropdown menu /
    /// combo box / select the default is to honor Escape. For
    /// modal shell the default is to honor both Escape and the
    /// scrim.
    #[default]
    Default,
    /// Always dismiss on the relevant event.
    Always,
    /// Never dismiss on the relevant event; the caller is
    /// responsible for closing the component.
    Never,
}

impl DismissBehavior {
    /// Resolve to the boolean used by components internally.
    /// `Default` returns `true` (dismiss on Escape) for the four
    /// popover-style components.
    pub fn resolve_escape(self) -> bool {
        match self {
            Self::Default | Self::Always => true,
            Self::Never => false,
        }
    }
}
