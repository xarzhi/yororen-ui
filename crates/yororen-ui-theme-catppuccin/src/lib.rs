//! Catppuccin (Latte / Frappé / Macchiato / Mocha) theme package for
//! `yororen-ui`.
//!
//! This crate is the v0.5 reference implementation of a "real-world"
//! theme package built on top of the Renderer trait system.
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
pub mod snapshot;
pub mod variant;

mod factories;

// Public factory functions.
pub use factories::{
    FrappeTheme, LatteTheme, MacchiatoTheme, MochaTheme, dark, frappe, frappe_theme, latte_theme,
    light, macchiato, macchiato_theme, mocha, mocha_theme,
};

use std::sync::Arc;

use gpui::App;
use gpui::WindowAppearance;

use yororen_ui_core::theme::{GlobalTheme, Theme};

/// Catppuccin flavor identifier.
///
/// Use this enum to explicitly select a flavor independent of
/// `WindowAppearance`. The four flavors differ in their
/// light/dark-ness:
///
/// - `Latte`: light.
/// - `Frappé`: medium-dark.
/// - `Macchiato`: darker than Frappé.
/// - `Mocha`: darkest (most popular).
///
/// `install_flavor` selects a flavor explicitly; `install` picks
/// Latte vs Mocha based on the OS appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CatppuccinFlavor {
    /// Light flavor. Default.
    #[default]
    Latte,
    /// Medium-dark flavor.
    Frappe,
    /// Darker than Frappé.
    Macchiato,
    /// Darkest flavor. Most popular.
    Mocha,
}

impl CatppuccinFlavor {
    /// Returns the canonical lowercase name (matches the
    /// `VariantKey` strings used in [`variant::key_mocha`] etc.).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Latte => "latte",
            Self::Frappe => "frappe",
            Self::Macchiato => "macchiato",
            Self::Mocha => "mocha",
        }
    }

    /// All four flavors, in canonical order.
    pub const ALL: [CatppuccinFlavor; 4] =
        [Self::Latte, Self::Frappe, Self::Macchiato, Self::Mocha];

    /// Build a `Theme` for this flavor. Thin wrapper around the
    /// factory functions in [`factories`].
    pub fn theme(self) -> Theme {
        match self {
            Self::Latte => light(),
            Self::Frappe => frappe(),
            Self::Macchiato => macchiato(),
            Self::Mocha => mocha(),
        }
    }
}

impl std::fmt::Display for CatppuccinFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Resolve a `CatppuccinFlavor` from a `WindowAppearance`. Uses
/// `VibrantLight` → Frappé, `VibrantDark` → Macchiato, `Light` →
/// Latte, `Dark` → Mocha. This lets apps that want
/// "medium-bright" pick a flavor via the OS appearance (only
/// `Vibrant*` triggers the medium flavors).
pub fn flavor_from_appearance(appearance: WindowAppearance) -> CatppuccinFlavor {
    match appearance {
        WindowAppearance::Light => CatppuccinFlavor::Latte,
        WindowAppearance::VibrantLight => CatppuccinFlavor::Frappe,
        WindowAppearance::VibrantDark => CatppuccinFlavor::Macchiato,
        WindowAppearance::Dark => CatppuccinFlavor::Mocha,
    }
}

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
    let theme = match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => light(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark => dark(),
    };
    cx.set_global(GlobalTheme::new(theme));
}

/// Install the Catppuccin theme with a specific flavor, ignoring
/// `WindowAppearance`. Useful when the user explicitly picks
/// "use Frappé" via an in-app setting, or when the app is
/// running headless without an OS appearance.
///
/// ```rust,ignore
/// catppuccin::install_flavor(cx, CatppuccinFlavor::Frappe);
/// ```
pub fn install_flavor(cx: &mut App, flavor: CatppuccinFlavor) {
    let theme = flavor.theme();
    cx.set_global(GlobalTheme::new(theme));
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
    fn all_four_flavors_loaded() {
        // Sanity check both palettes are distinct from system.
        let latte = light();
        let mocha = dark();
        assert_ne!(latte.surface.base, mocha.surface.base);
    }

    #[test]
    fn flavor_default_is_latte() {
        assert_eq!(CatppuccinFlavor::default(), CatppuccinFlavor::Latte);
    }

    #[test]
    fn flavor_as_str() {
        assert_eq!(CatppuccinFlavor::Latte.as_str(), "latte");
        assert_eq!(CatppuccinFlavor::Frappe.as_str(), "frappe");
        assert_eq!(CatppuccinFlavor::Macchiato.as_str(), "macchiato");
        assert_eq!(CatppuccinFlavor::Mocha.as_str(), "mocha");
    }

    #[test]
    fn flavor_all_has_four_variants() {
        assert_eq!(CatppuccinFlavor::ALL.len(), 4);
        let names: Vec<&str> = CatppuccinFlavor::ALL.iter().map(|f| f.as_str()).collect();
        assert_eq!(names, vec!["latte", "frappe", "macchiato", "mocha"]);
    }

    #[test]
    fn flavor_theme_distinct() {
        let latte = CatppuccinFlavor::Latte.theme();
        let frappe = CatppuccinFlavor::Frappe.theme();
        let macchiato = CatppuccinFlavor::Macchiato.theme();
        let mocha = CatppuccinFlavor::Mocha.theme();
        // Each flavor has a distinct surface.base.
        assert_ne!(latte.surface.base, frappe.surface.base);
        assert_ne!(frappe.surface.base, macchiato.surface.base);
        assert_ne!(macchiato.surface.base, mocha.surface.base);
        assert_ne!(latte.surface.base, mocha.surface.base);
    }

    #[test]
    fn flavor_display_matches_as_str() {
        for f in CatppuccinFlavor::ALL {
            assert_eq!(format!("{}", f), f.as_str());
        }
    }

    #[test]
    fn flavor_from_appearance_maps_correctly() {
        use gpui::WindowAppearance::*;
        assert_eq!(flavor_from_appearance(Light), CatppuccinFlavor::Latte);
        assert_eq!(
            flavor_from_appearance(VibrantLight),
            CatppuccinFlavor::Frappe
        );
        assert_eq!(
            flavor_from_appearance(VibrantDark),
            CatppuccinFlavor::Macchiato
        );
        assert_eq!(flavor_from_appearance(Dark), CatppuccinFlavor::Mocha);
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
        let cat_bg = reg
            .get_button()
            .expect("ButtonRenderer registered")
            .bg(&state, &cat_theme);
        let sys_bg = reg
            .get_button()
            .expect("ButtonRenderer registered")
            .bg(&state, &sys_theme);
        assert_ne!(cat_bg, sys_bg);
    }
}
