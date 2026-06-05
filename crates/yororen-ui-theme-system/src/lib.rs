//! Default light/dark theme for `yororen-ui`.
//!
//! This crate provides a neutral, no-brand palette. Third-party theme
//! packages (e.g. `yororen-ui-theme-catppuccin`) can implement the same
//! install pattern by depending on `yororen-ui-core` and providing their
//! own `light()`/`dark()` factories.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use yororen_ui::theme_system;
//!
//! theme_system::install(cx, window.appearance());
//! ```

use std::sync::Arc;

use gpui::App;
use gpui::WindowAppearance;

use yororen_ui_renderer::theme::{GlobalTheme, Theme};

mod dark;
mod light;

/// Install the default theme on the given `App`. Picks light or dark based
/// on the current `WindowAppearance`.
///
/// Use this once during app bootstrap:
/// ```rust,ignore
/// theme_system::install(cx, window.appearance());
/// ```
///
/// The single supported identity: a `GlobalTheme` global. The
/// previous `ThemeSet` (light/dark factory) was removed; the app
/// now picks the right `Theme` for the OS appearance and stores it
/// in a single `GlobalTheme` global.
pub fn install(cx: &mut App, appearance: WindowAppearance) {
    let theme = match appearance {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => dark::dark(),
        WindowAppearance::Light | WindowAppearance::VibrantLight => light::light(),
    };
    cx.set_global(GlobalTheme::new(theme));
}

/// Build a `Theme` from the system's default light palette.
pub fn light() -> Theme {
    light::light()
}

/// Build a `Theme` from the system's default dark palette.
pub fn dark() -> Theme {
    dark::dark()
}

/// Convenience: a default light `Arc<Theme>`.
pub fn light_arc() -> Arc<Theme> {
    Arc::new(light::light())
}

/// Convenience: a default dark `Arc<Theme>`.
pub fn dark_arc() -> Arc<Theme> {
    Arc::new(dark::dark())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_dark_distinct() {
        let l = light();
        let d = dark();
        assert_ne!(l.surface.canvas.a, d.surface.canvas.a + 1.0); // sanity: distinct a channel
    }
}
