//! yororen-ui Theme Compare Demo
//!
//! Verifies Phase D end-to-end. Splits the window into a left half
//! (theme-system light) and a right half (theme-mini indigo/cyan). The
//! same Button / Card / Modal renderers are used on both sides — only
//! the `Theme.renderers` registry differs.
//!
//! The "Apply mini to left" button flips the left half to the mini
//! theme at runtime, proving that `RendererRegistry` is a per-Theme
//! swappable handle, not a process-global.

use std::sync::Arc;

use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::theme::{GlobalTheme, Theme, ThemeSet};

use yororen_ui_theme_system as theme_system;
use yororen_ui_theme_mini as theme_mini;

mod compare_app;
mod state;

use state::ThemeCompareState;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        yororen_ui::component::init(cx);
        // Default install uses theme-system; the right half of the
        // window overrides to the mini registry on the first render.
        theme_system::install(cx, cx.window_appearance());

        let st = ThemeCompareState::new(cx);
        cx.set_global(st);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(820.0), px(440.0)),
                cx,
            ))),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| cx.new(compare_app::ThemeCompareApp::new));
    });
}

/// Build a `Theme` from `theme-system::light()` but with `renderers`
/// swapped to the mini registry. Used to populate the right half
/// of the window with the indigo / cyan skin.
pub fn mini_theme() -> Theme {
    let mut t = theme_system::light();
    t.renderers = theme_mini::mini_registry();
    t
}

pub fn system_light_theme() -> Theme {
    theme_system::light()
}

pub fn system_dark_theme() -> Theme {
    theme_system::dark()
}

/// Replace the active `GlobalTheme` with a new one wrapping `theme`.
pub fn set_active_theme(cx: &mut App, theme: Theme) {
    cx.set_global(GlobalTheme::new_with_themes(
        gpui::WindowAppearance::Light,
        ThemeSet::new(theme),
    ));
    cx.refresh_windows();
}

pub fn current_theme(cx: &App) -> Arc<Theme> {
    cx.global::<GlobalTheme>().current().clone()
}
