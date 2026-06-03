//! yororen-ui Variant Showcase Demo
//!
//! Verifies Phase E end-to-end. Registers two custom button variants
//! ("ghost" and "branded") via the global `VariantRegistry`, then
//! builds a small button gallery where each row uses a different
//! variant. The 3 built-in variants (Neutral / Primary / Danger) are
//! rendered through the existing `TokenButtonRenderer`; the 2 custom
//! variants are resolved at render time through the registry.

use std::sync::Arc;

use gpui::{App, AppContext, Application, Hsla, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::renderer::{
    GlobalVariantRegistry, VariantKey, VariantRegistry, VariantState, VariantStyle,
};

use yororen_ui_locale_en as locale_en;
use yororen_ui_theme_system as theme_system;

mod variant_app;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        yororen_ui::component::init(cx);
        theme_system::install(cx, cx.window_appearance());
        locale_en::install(cx);

        // Register the 2 custom variants the gallery uses.
        let reg = VariantRegistry::with_defaults();
        reg.register(VariantKey::borrowed("ghost"), Arc::new(GhostVariant));
        reg.register(VariantKey::borrowed("branded"), Arc::new(BrandedVariant));
        cx.set_global(GlobalVariantRegistry(Arc::new(reg)));

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(640.0), px(560.0)),
                cx,
            ))),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| cx.new(variant_app::VariantShowcaseApp::new));
    });
}

// ---------------------------------------------------------------------------
// Custom variant implementations. These are pure-data structs that
// implement the public `VariantStyle` trait. Third parties (your app,
// your theme package) can ship as many of these as they want and
// register them under any string key.
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct GhostVariant;

impl VariantStyle for GhostVariant {
    fn bg(&self, state: &VariantState) -> Hsla {
        if state.disabled {
            gpui::rgb(0xF4F4F5).into()
        } else {
            // Transparent — emulated by matching surface.
            gpui::rgb(0x000000).into()
        }
    }
    fn fg(&self, state: &VariantState) -> Hsla {
        if state.disabled {
            gpui::rgb(0xA1A1AA).into()
        } else {
            gpui::rgb(0x18181B).into()
        }
    }
    fn border(&self, state: &VariantState) -> Option<Hsla> {
        if state.disabled {
            Some(gpui::rgb(0xE4E4E7).into())
        } else {
            Some(gpui::rgb(0xD4D4D8).into())
        }
    }
    fn disabled_opacity(&self) -> f32 {
        1.0
    }
}

#[derive(Debug)]
struct BrandedVariant;

impl VariantStyle for BrandedVariant {
    fn bg(&self, state: &VariantState) -> Hsla {
        if state.disabled {
            gpui::rgb(0xFED7AA).into()
        } else {
            // Bold orange brand color.
            gpui::rgb(0xF97316).into()
        }
    }
    fn fg(&self, state: &VariantState) -> Hsla {
        if state.disabled {
            gpui::rgb(0x9A3412).into()
        } else {
            gpui::rgb(0xFFFFFF).into()
        }
    }
    fn border(&self, _state: &VariantState) -> Option<Hsla> {
        None
    }
    fn disabled_opacity(&self) -> f32 {
        1.0
    }
}
