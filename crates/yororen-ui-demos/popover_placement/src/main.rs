//! Popover Placement Demo
//!
//! This demo verifies the `desired_menu_left` layout calculation and
//! `PopoverPlacement::BottomEnd` behavior.
//!
//! ## What to check
//! - **BottomStart** popover aligns its start edge to the trigger's start edge
//! - **BottomEnd** popover aligns its end edge to the trigger's end edge
//! - Switching between LTR (en) and RTL (ar) flips the alignment correctly
//! - Narrow window forces clamping so the menu stays within bounds
//!
//! ## Run
//! ```bash
//! cd demo/popover_placement && cargo run
//! ```

mod popover_app;

use gpui::{App, AppContext, Application, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::component;
use yororen_ui::i18n::{I18n, Locale};
use yororen_ui::theme::GlobalTheme;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        component::init(cx);
        cx.set_global(GlobalTheme::new(cx.window_appearance()));
        cx.set_global(I18n::with_embedded(Locale::new("en").unwrap()));

        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(640.0), px(400.0)),
                cx,
            ))),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| cx.new(popover_app::PopoverPlacementApp::new));
    });
}
