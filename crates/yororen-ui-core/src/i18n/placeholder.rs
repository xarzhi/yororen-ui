//! Placeholder text resolution for components.
//!
//! `core` does not ship built-in placeholder strings; it only defines the
//! [`PlaceholderKey`] enum (so components can ask "what placeholder do you
//! want for this slot?") and a [`PlaceholderResolver`] trait that
//! applications implement to provide text from any source (i18n catalog,
//! app config, hard-coded English, ...).
//!
//! # Usage
//!
//! Components built in `core` always carry a sensible English default and
//! expose a `.placeholder(SharedString)` builder. To override the default
//! globally (e.g. localized strings) without touching every call-site,
//! register a [`PlaceholderResolver`] once at app startup:
//!
//! ```ignore
//! use std::sync::Arc;
//! use yororen_ui::i18n::{
//!     GlobalPlaceholderResolver, PlaceholderKey, PlaceholderResolver,
//! };
//!
//! struct MyResolver;
//! impl PlaceholderResolver for MyResolver {
//!     fn resolve(&self, key: PlaceholderKey) -> Option<SharedString> {
//!         match key {
//!             PlaceholderKey::Select => Some("Pick one…".into()),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! cx.set_global(GlobalPlaceholderResolver(Arc::new(MyResolver)));
//! ```
//!
//! A component checks the global resolver first, then its own default.
//! A user-supplied `.placeholder(...)` always wins.

use std::sync::Arc;

use gpui::{App, Global, SharedString};

/// Identifies a placeholder slot inside a core component.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PlaceholderKey {
    /// The "no value selected" hint shown by `select::Select`.
    Select,
    /// The "type to filter" hint shown by `combo_box::ComboBox`.
    ComboBoxSearch,
    /// The "press a key combination" hint shown by `keybinding_input::KeybindingInput`.
    KeybindingPressKeys,
    /// The "waiting for keys" hint shown while `KeybindingInput` is in capture mode.
    KeybindingWaiting,
    /// The "no file selected" hint shown by `file_path_input::FilePathInput`.
    FilePath,
}

/// Resolves placeholder text for a given slot.
///
/// Implementations are registered as a single global
/// [`GlobalPlaceholderResolver`]. Returning `None` means "no opinion" — the
/// component falls back to its built-in default (typically English).
pub trait PlaceholderResolver: Send + Sync {
    fn resolve(&self, key: PlaceholderKey) -> Option<SharedString>;
}

/// A no-op resolver; useful as a default and in tests.
pub struct NoopPlaceholderResolver;

impl PlaceholderResolver for NoopPlaceholderResolver {
    fn resolve(&self, _key: PlaceholderKey) -> Option<SharedString> {
        None
    }
}

/// Global wrapper used to register a [`PlaceholderResolver`] on the app.
pub struct GlobalPlaceholderResolver(pub Arc<dyn PlaceholderResolver>);

impl Global for GlobalPlaceholderResolver {}

/// Convenience: read a placeholder from the global resolver, if any.
pub trait PlaceholderContext {
    fn placeholder(&self, key: PlaceholderKey) -> Option<SharedString>;
}

impl PlaceholderContext for App {
    fn placeholder(&self, key: PlaceholderKey) -> Option<SharedString> {
        self.try_global::<GlobalPlaceholderResolver>()
            .and_then(|r| r.0.resolve(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TwoPlaceholder;
    impl PlaceholderResolver for TwoPlaceholder {
        fn resolve(&self, key: PlaceholderKey) -> Option<SharedString> {
            match key {
                PlaceholderKey::Select => Some("Choose…".into()),
                PlaceholderKey::ComboBoxSearch => Some("Type…".into()),
                _ => None,
            }
        }
    }

    #[test]
    fn noop_returns_none() {
        let r = NoopPlaceholderResolver;
        assert!(r.resolve(PlaceholderKey::Select).is_none());
    }

    #[test]
    fn custom_resolver_returns_overrides() {
        let r = TwoPlaceholder;
        assert_eq!(
            r.resolve(PlaceholderKey::Select).map(|s| s.to_string()),
            Some("Choose…".to_string())
        );
        assert_eq!(
            r.resolve(PlaceholderKey::ComboBoxSearch)
                .map(|s| s.to_string()),
            Some("Type…".to_string())
        );
        assert!(r.resolve(PlaceholderKey::FilePath).is_none());
    }
}
