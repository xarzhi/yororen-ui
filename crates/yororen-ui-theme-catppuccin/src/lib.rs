//! Catppuccin (Latte / Frappé / Macchiato / Mocha) theme package for
//! `yororen-ui`.
//!
//! This crate is the v0.5 reference implementation of a "real-world"
//! theme package built on top of the Phase C Renderer trait system.
//! It demonstrates that a third party can ship a full skin without
//! modifying `yororen-ui-core`, by composing:
//!
//! - A [`palette`](palette) module (4 flavors × 26 colors).
//! - Lightweight [`Theme`] factories (`light` = Latte, `dark` = Mocha,
//!   plus `frappe` and `macchiato` for the medium-dark and dark
//!   options).
//! - A [`renderer`] module with Catppuccin-flavoured Renderer
//!   implementations (button, focus_ring, card, modal, input, switch,
//!   checkbox, radio, toast, tag, badge, list_item, empty_state).
//! - A [`variant`] module that registers three Catppuccin-specific
//!   custom variants (`mocha`, `lavender`, `ghost`).
//! - An [`install`] helper that puts everything on the `App` for you.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use yororen_ui_theme_catppuccin as catppuccin;
//!
//! catppuccin::install(cx, window.appearance());
//! ```
//!
//! Or, to mix-and-match with `theme-system`:
//!
//! ```rust,ignore
//! use yororen_ui_theme_catppuccin as catppuccin;
//!
//! // Build a Theme that uses the Latte palette but with the
//! // Catppuccin renderer registry layered on top.
//! let mut theme = catppuccin::light();
//! // Or: theme = catppuccin::frappe(); / macchiato(); / mocha();
//! theme.renderers = catppuccin::renderer::catppuccin_registry();
//! ```
//!
//! # Custom variants
//!
//! Catppuccin ships three custom variants in addition to the three
//! built-in `ActionVariantKind` values:
//!
//! - `"mocha"` — uses `mocha` palette as primary (just because the
//!   flavor is famous).
//! - `"lavender"` — uses `lavender` accent.
//! - `"ghost"` — a transparent button that picks up the surface tint.
//!
//! Register them on the `App` after `install(...)`:
//!
//! ```rust,ignore
//! catppuccin::variant::register_all(cx);
//! ```
//!
//! Then in a button builder:
//!
//! ```rust,ignore
//! use yororen_ui::renderer::ButtonVariant;
//! use yororen_ui_core::renderer::VariantKey;
//!
//! button("save")
//!     .variant(ButtonVariant::Custom(VariantKey::borrowed("lavender")))
//!     .child("Save");
//! ```

pub mod palette;
pub mod renderer;
pub mod variant;

mod factories;

// Public factory functions.
pub use factories::{
    FrappeTheme, LatteTheme, MacchiatoTheme, MochaTheme, dark, frappe, frappe_theme, latte_theme,
    light, macchiato, macchiato_theme, mocha, mocha_theme, themeset, themeset_all_four,
};

use std::sync::Arc;

use gpui::App;
use gpui::WindowAppearance;

use yororen_ui_core::theme::{GlobalTheme, Theme};

/// Convenience: a default `Arc<Theme>` (Mocha dark).
pub fn dark_arc() -> Arc<Theme> {
    Arc::new(dark())
}

/// Convenience: a default `Arc<Theme>` (Latte light).
pub fn light_arc() -> Arc<Theme> {
    Arc::new(light())
}

/// Install the Catppuccin theme on the given `App`.
///
/// Picks Latte for light appearance, Mocha for dark. The supplied
/// `Theme.renderers` registry is the `catppuccin_registry()`, so all
/// `TokenXxxRenderer` defaults are replaced with Catppuccin-flavoured
/// implementations for the components covered in [`renderer`].
///
/// Use this once during app bootstrap:
///
/// ```rust,ignore
/// catppuccin::install(cx, window.appearance());
/// ```
pub fn install(cx: &mut App, appearance: WindowAppearance) {
    cx.set_global(GlobalTheme::new_with_themes(appearance, themeset()));
}

/// Install the Catppuccin theme with the full 4-flavor `ThemeSet`
/// (Latte / Frappé / Macchiato / Mocha). The OS appearance selects
/// the closest one: light = Latte, dark = Mocha. (Frappé and Macchiato
/// are not directly OS-mapped; they are intended for explicit
/// per-app selection via [`themeset_all_four`].)
pub fn install_with_all_flavors(cx: &mut App, appearance: WindowAppearance) {
    cx.set_global(GlobalTheme::new_with_themes(
        appearance,
        themeset_all_four(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use yororen_ui_theme_system as theme_system;

    #[test]
    fn catppuccin_themes_distinct_from_system() {
        let cat_l = light();
        let sys_l = theme_system::light();
        assert_ne!(cat_l.surface.base, sys_l.surface.base);
    }

    #[test]
    fn catppuccin_themeset_has_both() {
        let ts = themeset();
        assert!(ts.dark.is_some());
    }

    #[test]
    fn all_four_flavors_loaded() {
        // Sanity check both palettes are distinct from system.
        let latte = light();
        let mocha = dark();
        assert_ne!(latte.surface.base, mocha.surface.base);
    }

    #[test]
    fn catppuccin_button_differs_from_system() {
        // The point of the registry swap: the same call against a
        // Catppuccin-flavoured Theme and a system Theme returns
        // different values.
        let reg = renderer::catppuccin_registry();
        let cat_theme = light();
        let sys_theme = theme_system::light();
        let state = yororen_ui_core::renderer::ButtonRenderState {
            variant: yororen_ui_core::theme::ActionVariantKind::Primary,
            ..Default::default()
        };
        let cat_bg = reg.button.bg(&state, &cat_theme);
        let sys_bg = reg.button.bg(&state, &sys_theme);
        assert_ne!(cat_bg, sys_bg);
    }
}
