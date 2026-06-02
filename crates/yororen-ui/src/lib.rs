//! Meta-crate for yororen-ui.
//!
//! Re-exports [`yororen_ui_core`] (headless primitives + i18n mechanism) and
//! [`yororen_ui_theme_system`] (default light/dark theme) for convenience.
//!
//! For full control over theming, depend on `yororen-ui-core` and `yororen-ui-theme-system`
//! directly.

pub use yororen_ui_core::*;
pub use yororen_ui_theme_system as theme_system;
