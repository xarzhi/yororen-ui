//! Meta-crate for yororen-ui.
//!
//! Re-exports [`yororen_ui_core`] (headless primitives + i18n mechanism),
//! [`yororen_ui_theme_system`] (default light/dark theme), and the bundled
//! locale catalogs (`en`, `zh-CN`, `ar`).
//!
//! For full control over theming and translation data, depend on
//! `yororen-ui-core`, `yororen-ui-theme-system`, and the individual
//! `yororen-ui-locale-*` crates directly.

pub use yororen_ui_core::*;
pub use yororen_ui_locale_ar as locale_ar;
pub use yororen_ui_locale_en as locale_en;
pub use yororen_ui_locale_zh_cn as locale_zh_cn;
pub use yororen_ui_theme_system as theme_system;
