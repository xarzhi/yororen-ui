//! yororen-ui Brutalism Renderer Demo
//!
//! A single window that exercises the 38 `XxxRenderer` impls
//! in the brutalism-renderer crate. Every component on screen
//! pulls colors from the bundled `brutalism-light.json` JSON
//! and the brutalist geometry (3px black borders, 0-radius,
//! 4px-Y offset shadow, monospace font) from the renderers'
//! hardcoded `style::BRUTAL_*` constants.
//!
//! The point of the demo: register a *different* renderer on
//! top of the default one and watch the same `headless::*`
//! factories render with a completely different visual
//! vocabulary. No headless code changes; the renderer swap is
//! the only knob.

use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui_brutalism_renderer as brutalism;
use yororen_ui_locale_en as locale_en;

mod brutalism_app;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Install the brutalism renderer with the bundled light
        // theme. This both sets the global Theme (loaded from
        // `themes/brutalism-light.json`) AND registers all 38
        // `BrutalXxxRenderer` impls against the core
        // `RendererRegistry`. After this call every
        // `headless::Xxx` factory's `.render(cx)` is
        // routed through the brutalist look.
        brutalism::install_with_default_theme(cx);
        locale_en::install(cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(900.0), px(1000.0)),
                cx,
            ))),
            ..Default::default()
        };
        let app_entity = cx.new(|_cx| brutalism_app::BrutalismApp::new());
        let _ = cx.open_window(options, |_, _cx| app_entity);
    });
}
