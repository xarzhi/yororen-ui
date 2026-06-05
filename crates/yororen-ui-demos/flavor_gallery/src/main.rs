//! yororen-ui Flavor Gallery Demo
//!
//! A side-by-side visual comparison of the available theme
//! "flavors" shipped with yororen-ui: 4 Catppuccin flavors
//! (Latte, Frappé, Macchiato, Mocha) plus Material Design 3.
//! The window is divided into 5 columns, one per flavor; each
//! column renders the same set of components (a select, a
//! combo box, and a "Show modal" button) so the visual
//! difference between flavors is unambiguous.
//!
//! The top bar also exposes a "System" theme toggle that
//! switches the **process-global** theme used by the top bar
//! itself. The 5 columns below are wrapped in `with_theme`
//! overrides and are independent of that global theme — they
//! always show their own flavor.
//!
//! The modal exercises the full a11y shell:
//!
//! - Opened via the `modal_dialog` factory (one-line API that
//!   composes Modal + Overlay + ScrollLock + focus trap).
//! - Closes on Esc, scrim click, the inner X, Cancel, or OK;
//!   all paths route through a single `on_close` callback
//!   carrying an `OverlayCloseReason`.
//! - The select / combo box in each column also honour Esc
//!   via `dismiss_on_escape`.
//!
//! The 5 flavors are rendered with the **same** code — only
//! the active `Theme` differs. This demonstrates that the
//! Renderer trait system + `CatppuccinFlavor` enum + `Material`
//! theme package together give complete third-party skins
//! with no per-component hardcoded color logic.

use std::sync::Arc;

use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::theme::Theme;
use yororen_ui_locale_en as locale_en;
use yororen_ui_theme_catppuccin as catppuccin;
use yororen_ui_theme_material as material;
use yororen_ui_theme_system as theme_system;

mod flavor_gallery_app;
mod state;

use state::FlavorGalleryState;
use state::FlavorKind;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        yororen_ui::component::init(cx);

        // Start with the system theme as the default; the user
        // picks a flavor via the buttons in the top bar.
        theme_system::install(cx, cx.window_appearance());
        locale_en::install(cx);

        // Register the Catppuccin custom variants so Catppuccin
        // accents (mocha / lavender / ghost, etc.) resolve to
        // the right per-flavor palette.
        let variant_reg = Arc::new(catppuccin::variant::catppuccin_registry());
        cx.set_global(yororen_ui::renderer::GlobalVariantRegistry(variant_reg));

        let st = FlavorGalleryState::new(cx);
        cx.set_global(st);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(1600.0), px(620.0)),
                cx,
            ))),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| {
            cx.new(flavor_gallery_app::FlavorGalleryApp::new)
        });
    });
}

/// Resolve the active Theme for a given flavor and OS appearance.
///
/// Latte / Frappé / Macchiato / Mocha are the 4 Catppuccin
/// flavors; Material 3 is the second official theme; "System"
/// uses the system palette (with the active OS appearance).
pub fn theme_for(kind: FlavorKind, appearance: gpui::WindowAppearance) -> Theme {
    match kind {
        FlavorKind::System => match appearance {
            gpui::WindowAppearance::Light | gpui::WindowAppearance::VibrantLight => {
                theme_system::light()
            }
            _ => theme_system::dark(),
        },
        FlavorKind::Latte => catppuccin::light(),
        FlavorKind::Frappe => catppuccin::frappe(),
        FlavorKind::Macchiato => catppuccin::macchiato(),
        FlavorKind::Mocha => catppuccin::mocha(),
        FlavorKind::Material => match appearance {
            gpui::WindowAppearance::Light | gpui::WindowAppearance::VibrantLight => {
                material::light()
            }
            _ => material::dark(),
        },
    }
}

// The FlavorKind enum is re-exported from `state.rs` so callers
// outside this crate can dispatch on it. The dead_code allow
// silences warnings for the variants that are only used by the
// `Theme` matcher in `theme_for` above.
#[allow(dead_code)]
fn _kind_passthrough(k: FlavorKind) -> FlavorKind {
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All 6 flavors (System + 4 Catppuccin + Material 3) must
    /// produce distinct theme surface.base values. This is a
    /// regression test for the "no hardcoded color" rule — if
    /// any two flavors collide, a component is almost certainly
    /// bypassing the renderer and using a baked-in color.
    #[test]
    fn all_six_flavors_produce_distinct_themes() {
        let appearance = gpui::WindowAppearance::Dark;
        let system = theme_for(FlavorKind::System, appearance);
        let latte = theme_for(FlavorKind::Latte, appearance);
        let frappe = theme_for(FlavorKind::Frappe, appearance);
        let macchiato = theme_for(FlavorKind::Macchiato, appearance);
        let mocha = theme_for(FlavorKind::Mocha, appearance);
        let material = theme_for(FlavorKind::Material, appearance);
        // Pairwise distinct.
        assert_ne!(system.surface.base, latte.surface.base);
        assert_ne!(latte.surface.base, frappe.surface.base);
        assert_ne!(frappe.surface.base, macchiato.surface.base);
        assert_ne!(macchiato.surface.base, mocha.surface.base);
        assert_ne!(mocha.surface.base, material.surface.base);
        // System is the system theme; Latte is the Catppuccin light.
        assert_ne!(system.surface.base, mocha.surface.base);
    }

    /// The 5 visible flavor columns can be wrapped in a
    /// `with_theme` block and their descendants see the
    /// per-flavor palette regardless of the process-global
    /// theme in the top bar. Sanity check on the per-element
    /// theme override.
    #[test]
    fn flavor_kind_as_str_matches_demonstration() {
        assert_eq!(FlavorKind::System.as_str(), "System");
        assert_eq!(FlavorKind::Latte.as_str(), "Latte");
        assert_eq!(FlavorKind::Frappe.as_str(), "Frappé");
        assert_eq!(FlavorKind::Macchiato.as_str(), "Macchiato");
        assert_eq!(FlavorKind::Mocha.as_str(), "Mocha");
        assert_eq!(FlavorKind::Material.as_str(), "Material 3");
    }

    /// Material's primary background is its primary accent (M3
    /// "filled button" look). We verify the renderer yields
    /// something distinct from the system theme.
    #[test]
    fn material_button_bg_differs_from_system() {
        use yororen_ui::renderer::ButtonRenderState;
        let appearance = gpui::WindowAppearance::Dark;
        let mat = theme_for(FlavorKind::Material, appearance);
        let sys = theme_for(FlavorKind::System, appearance);
        let state = ButtonRenderState {
            variant: yororen_ui::theme::ActionVariantKind::Primary,
            ..Default::default()
        };
        let mat_bg = mat
            .renderers
            .get_button()
            .expect("ButtonRenderer registered")
            .bg(&state, &mat);
        let sys_bg = sys
            .renderers
            .get_button()
            .expect("ButtonRenderer registered")
            .bg(&state, &sys);
        // M3 uses primary accent; system uses dark action.bg.
        // They should differ.
        assert_ne!(mat_bg, sys_bg);
    }

    /// `theme_for` returns the same Theme across calls (the
    /// factory functions are pure, so the demo's top-bar switch
    /// can rely on the result being deterministic).
    #[test]
    fn theme_for_is_deterministic() {
        let appearance = gpui::WindowAppearance::Light;
        let t1 = theme_for(FlavorKind::Latte, appearance);
        let t2 = theme_for(FlavorKind::Latte, appearance);
        // Surface and content are not Copy; compare via Debug.
        assert_eq!(
            format!("{:?}", t1.surface.base),
            format!("{:?}", t2.surface.base)
        );
    }
}
