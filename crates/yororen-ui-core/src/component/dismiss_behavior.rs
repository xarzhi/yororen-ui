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
    /// Use the component's own default. Each component decides
    /// what its default dismiss behavior is and passes that as
    /// the `default_dismiss` argument to [`Self::resolve`].
    #[default]
    Default,
    /// Force dismissal on the relevant event regardless of the
    /// component's own default.
    Always,
    /// Never dismiss on the relevant event; the caller is
    /// responsible for closing the component.
    Never,
}

impl DismissBehavior {
    /// Resolve to the boolean used by components internally.
    ///
    /// `default_dismiss` is the component-specific default
    /// (e.g. popover/dropdown menu/select/combo box typically
    /// pass `true` for Escape; some modal variants pass `false`
    /// for scrim clicks). This is what makes `Default` and
    /// `Always` genuinely distinct: `Default` defers to the
    /// caller's policy, `Always` overrides it.
    pub fn resolve(self, default_dismiss: bool) -> bool {
        match self {
            Self::Default => default_dismiss,
            Self::Always => true,
            Self::Never => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_defers_to_caller_default() {
        assert!(DismissBehavior::Default.resolve(true));
        assert!(!DismissBehavior::Default.resolve(false));
    }

    #[test]
    fn always_overrides_caller_default() {
        assert!(DismissBehavior::Always.resolve(true));
        assert!(DismissBehavior::Always.resolve(false));
    }

    #[test]
    fn never_overrides_caller_default() {
        assert!(!DismissBehavior::Never.resolve(true));
        assert!(!DismissBehavior::Never.resolve(false));
    }
}
