//! yororen-ui Theme Showcase Demo
//!
//! A single window with 4 rows. Each row labels a theme
//! (system-light, system-dark, catppuccin-mocha,
//! material-rose) and shows a `headless::button` rendered
//! by the same `TokenButtonRenderer` with the *currently
//! active* global theme. Switching the active theme (via
//! `cx.install_theme(...)`) instantly recolors every row.
//!
//! The point of the demo: themes are **just JSON**. The
//! renderer doesn't care which JSON you feed it.

use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};

use yororen_ui::Theme;
use yororen_ui::assets::UiAsset;
use yororen_ui::theme as theme_mod;
use yororen_ui_default_renderer as default_renderer;

mod theme_app;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Install the default renderer + the system-light
        // theme. The user can press a key (e.g. F5) to cycle
        // through themes, but for the demo we just show all
        // 4 themes in the layout below.
        default_renderer::install(cx, cx.window_appearance());
        // The four themes bundled for the demo:
        theme_mod::install(cx, Theme::from_json(SYSTEM_LIGHT).unwrap());
        // (The user can swap any of the four `THEME_*` strings
        // here to see a different palette.)

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(700.0), px(500.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(|_cx| theme_app::ThemeApp::new()));
    });
}

// Theme JSONs are inlined as `&str` constants so the demo
// is self-contained. (The `theme_showcase/themes/` dir also
// has copies as JSON files for the `include_str!` path.)
const SYSTEM_LIGHT: &str = r##"{
  "action": { "primary": { "bg": "#121214", "fg": "#ffffff" } },
  "surface": { "base": "#FFFFFF" },
  "content": { "primary": "#141416" }
}"##;
