//! yororen-ui Theme Compare Demo
//!
//! Verifies Phase D end-to-end. Splits the window into a left half
//! (theme-system) and a right half (theme-mini). The same Button /
//! Card renderers are used on both sides — only the
//! `Theme.renderers` registry differs.
//!
//! Both halves track the current `WindowAppearance`: the system
//! half follows the OS light/dark setting, and the mini half
//! combines the matching light/dark palette with the mini
//! registry. Switching the OS appearance (or rebuilding the app
//! after a change) updates both halves.
//!
//! The right half is wrapped in `with_theme(right_theme, ...)`
//! from `yororen_ui::component`, which temporarily installs the
//! supplied theme as the global theme for the lifetime of the
//! right-half element. The "Switch right" button toggles which
//! theme that override uses, proving that `RendererRegistry` is a
//! per-Theme swappable handle, not a process-global.

use gpui::{App, AppContext, Application, WindowAppearance, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;

use yororen_ui_theme_system as theme_system;
use yororen_ui_theme_mini as theme_mini;
use yororen_ui::theme::Theme;

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

/// Resolve the system palette for the current OS appearance.
fn system_palette(appearance: WindowAppearance) -> Theme {
    match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => theme_system::light(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark => theme_system::dark(),
    }
}

/// Build a system theme matching the current `WindowAppearance`.
/// Wraps the matching light/dark palette from `theme-system` with
/// its default `RendererRegistry`. Used to populate the right
/// half of the window when its override is "system" — the result
/// tracks the OS's light/dark setting.
pub fn system_theme(appearance: WindowAppearance) -> Theme {
    system_palette(appearance)
}

/// Build a mini theme matching the current `WindowAppearance`.
/// Wraps the matching light/dark palette from `theme-system` and
/// swaps in the mini renderer registry. Used to populate the
/// right half of the window when its override is "mini".
pub fn mini_theme(appearance: WindowAppearance) -> Theme {
    let mut t = system_palette(appearance);
    t.renderers = theme_mini::mini_registry();
    t
}
