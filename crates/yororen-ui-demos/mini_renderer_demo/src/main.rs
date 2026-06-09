//! yororen-ui Mini Renderer Demo
//!
//! Mini-styled component gallery. Every button / icon-button
//! / toggle-button / label in this window is rendered by
//! `yororen-ui-mini-renderer`, which only reads
//! `themeColor` + `accentColor` from the theme JSON.
//!
//! The demo depends on `yororen-ui-mini-renderer` directly
//! because the `mini` feature on the `yororen-ui` meta is
//! optional. This shows the third-party-renderer install
//! pattern: the default renderer registers its 38 impls,
//! then the mini renderer registers its 4 overrides.

use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui_default_renderer as default_renderer;
use yororen_ui_mini_renderer as mini;

mod mini_app;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Default renderer first (38 token-styled components).
        default_renderer::install(cx, cx.window_appearance());
        // Mini on top (4 overrides: Button / IconButton /
        // ToggleButton / Label). Last-registered wins.
        mini::install(cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(560.0), px(360.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(|_cx| mini_app::MiniApp::new()));
    });
}
