//! yororen-ui Theme Showcase Demo
//!
//! Phase F.4 end-to-end proof. Renders a split window:
//!
//! - **Left half** uses the default `theme-system` palette + token
//!   renderers. (v0.5 visual baseline.)
//! - **Right half** uses the `yororen-ui-theme-catppuccin` Mocha
//!   palette + Catppuccin-flavoured renderers. The two halves share
//!   the same UI, so a side-by-side diff is unmistakable.
//!
//! The right half is wrapped in `with_theme(right_theme, ...)` so
//! its components pick up the Catppuccin theme without touching the
//! process-global theme. A "Switch right" button toggles which theme
//! the right half uses (system / catppuccin), proving that
//! `Theme.renderers` is a per-Theme swappable handle, not a
//! process-global.
//!
//! Also registers the three Catppuccin-specific custom variants
//! (`mocha`, `lavender`, `ghost`) on the global `VariantRegistry`,
//! and renders a "Custom variants" row that uses them.

use std::sync::Arc;

use gpui::{App, AppContext, Application, WindowAppearance, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::renderer::{GlobalVariantRegistry, RendererRegistry, VariantRegistry};

use yororen_ui_locale_en as locale_en;
use yororen_ui_theme_catppuccin as catppuccin;
use yororen_ui_theme_system as theme_system;

mod showcase_app;
mod state;

use state::ThemeShowcaseState;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        yororen_ui::component::init(cx);
        // Default install uses theme-system. The right half of the
        // window overrides to the Catppuccin theme via `with_theme`.
        theme_system::install(cx, cx.window_appearance());
        locale_en::install(cx);

        // Register the 3 Catppuccin custom variants so the gallery
        // can render them.
        let reg = Arc::new(catppuccin::variant::catppuccin_registry());
        cx.set_global(GlobalVariantRegistry(reg));

        let st = ThemeShowcaseState::new(cx);
        cx.set_global(st);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(900.0), px(620.0)),
                cx,
            ))),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| cx.new(showcase_app::ThemeShowcaseApp::new));
    });
}

/// Resolve the system palette for the current OS appearance, but
/// swap the renderer registry for the Catppuccin one. This proves
/// that a Catppuccin "look" can be applied to any palette, and
/// vice versa.
pub fn catppuccin_renderer_only(appearance: WindowAppearance) -> yororen_ui::theme::Theme {
    let mut t = match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => theme_system::light(),
        _ => theme_system::dark(),
    };
    t.renderers = RendererRegistry::token_based()
        .with_button(Arc::new(catppuccin::renderer::CatppuccinButtonRenderer))
        .with_card(Arc::new(catppuccin::renderer::CatppuccinCardRenderer))
        .with_modal(Arc::new(catppuccin::renderer::CatppuccinModalRenderer))
        .with_focus_ring(Arc::new(catppuccin::renderer::CatppuccinFocusRingRenderer));
    t
}

/// Build a Catppuccin theme matching the current `WindowAppearance`
/// (Latte for light, Mocha for dark) with the full Catppuccin
/// renderer registry.
pub fn catppuccin_theme(appearance: WindowAppearance) -> yororen_ui::theme::Theme {
    match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => catppuccin::light(),
        _ => catppuccin::dark(),
    }
}

/// Build a system theme matching the current `WindowAppearance`
/// using the v0.5 token-based defaults.
pub fn system_theme(appearance: WindowAppearance) -> yororen_ui::theme::Theme {
    match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => theme_system::light(),
        _ => theme_system::dark(),
    }
}

/// Sanity-check helper used in tests: the variant registry should
/// have all three Catppuccin variants registered.
pub fn assert_variants_registered(cx: &App) {
    let reg = cx.global::<GlobalVariantRegistry>();
    let _ = reg
        .0
        .resolve(&yororen_ui::renderer::VariantKey::borrowed("mocha"));
    let _ = reg
        .0
        .resolve(&yororen_ui::renderer::VariantKey::borrowed("lavender"));
    let _ = reg
        .0
        .resolve(&yororen_ui::renderer::VariantKey::borrowed("ghost"));
}

// We rely on the global VariantRegistry being present in
// `cx.global::<GlobalVariantRegistry>()`. This function is unused at
// the moment but kept so future demos can re-register without
// reaching into the global directly.
#[allow(dead_code)]
pub fn fresh_variant_registry() -> Arc<VariantRegistry> {
    Arc::new(catppuccin::variant::catppuccin_registry())
}
