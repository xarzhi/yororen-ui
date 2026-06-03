//! Meta-crate for yororen-ui.
//!
//! Re-exports [`yororen_ui_core`] (headless primitives + i18n mechanism),
//! [`yororen_ui_theme_system`] (default light/dark theme), and the bundled
//! locale catalogs (`en`, `zh-CN`, `ar`).
//!
//! Optional features:
//!
//! - `catppuccin` (default) — re-export the Catppuccin theme package as
//!   `theme_catppuccin`.
//! - `material` (default) — re-export the Material Design 3 theme
//!   package as `theme_material`.
//!
//! Disable defaults with `default-features = false` and pick only the
//! themes / locales you need.
//!
//! For full control over theming and translation data, depend on
//! `yororen-ui-core`, `yororen-ui-theme-system`, and the individual
//! `yororen-ui-locale-*` crates directly.

pub use yororen_ui_core::*;
pub use yororen_ui_locale_ar as locale_ar;
pub use yororen_ui_locale_en as locale_en;
pub use yororen_ui_locale_zh_cn as locale_zh_cn;
pub use yororen_ui_theme_system as theme_system;

#[cfg(feature = "catppuccin")]
pub use yororen_ui_theme_catppuccin as theme_catppuccin;

#[cfg(feature = "material")]
pub use yororen_ui_theme_material as theme_material;
