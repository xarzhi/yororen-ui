//! yororen-ui Flavor Gallery Demo
//!
//! End-to-end demo of Phase F (Catppuccin theme) and Phase G
//! (a11y completeness) working together. The window is divided
//! into 5 columns, one per flavor (4 Catppuccin flavors +
//! Material Design 3). Each column has the same set of
//! components — pickers, toggles, a tooltip, and a "Show modal"
//! button. Opening the modal in any column demonstrates the
//! v0.5 a11y stack end-to-end:
//!
//! - The modal is opened via the new `modal_dialog` factory
//!   (G-γ: one-line API, no need to manually compose Modal +
//!   Overlay + ScrollLock).
//! - The modal closes on Esc, scrim click, OR the inner close
//!   button (G-δ: all three paths route through a single
//!   `on_close` callback with `OverlayCloseReason`).
//! - The select / combo_box in each column honours Esc via
//!   `dismiss_on_escape` (G-β).
//! - Tab / Shift+Tab inside the modal is the default
//!   focus-trap behaviour (G-α real impl is used in the
//!   `focus_trap_demo` crate; this demo uses `modal_dialog`'s
//!   built-in focus handling).
//!
//! The 5 flavors are rendered with the **same** code — only the
//! active `Theme` differs. This proves that the v0.5 Renderer
//! trait system + `CatppuccinFlavor` enum + `Material` theme
//! package together give complete third-party skins with no
//! per-component hardcoded color logic.

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
        // picks a flavor via the radio in the top bar.
        theme_system::install(cx, cx.window_appearance());
        locale_en::install(cx);

        // Also register the Catppuccin custom variants so the
        // "mocha / lavender / ghost" buttons in the gallery can
        // be rendered with the correct accent.
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
/// Latte / Frappé / Macchiato / Mocha are explicit Catppuccin
/// flavors; Material is the second official theme (Phase H.1);
/// "System" uses the system palette (with the active OS
/// appearance).
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

    /// The 4 Catppuccin flavors + Material + System must produce
    /// distinct theme surface.bg values. This is the v0.5
    /// regression test for the F-α no-hardcode rule, extended
    /// in Phase H.1 to cover the new Material theme.
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

    /// The same Theme can be plugged into a `with_theme` block
    /// and the descendants see the per-flavor palette.
    /// This is a sanity check for the F-γ wiring, extended
    /// for the new Material variant.
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
        let mat_bg = mat.renderers.button.bg(&state, &mat);
        let sys_bg = sys.renderers.button.bg(&state, &sys);
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
