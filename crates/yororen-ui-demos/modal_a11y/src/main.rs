//! yororen-ui Modal a11y Demo
//!
//! Phase G.5 end-to-end proof. The window shows a few buttons that
//! each open a different kind of modal. Together they exercise the
//! v0.5 accessibility stack:
//!
//! - **Standard modal** ("Open standard modal"): opens a modal
//!   wrapped in [`Overlay`]. Closes on Escape, scrim click, or the
//!   Cancel / OK buttons. Body scroll is locked while open.
//! - **Non-dismissable modal** ("Open required modal"): opens a
//!   modal that does NOT close on Escape or scrim click. The user
//!   must click one of the action buttons to dismiss it.
//! - **Modal with scroll lock disabled** ("Open no-scroll-lock"):
//!   same as the standard modal but `.lock_scroll(false)` — useful
//!   for non-modal overlays where the user should still be able to
//!   scroll the page underneath.

use gpui::{
    App, AppContext, Application, WindowBounds, WindowOptions, px, size,
};

use yororen_ui::assets::UiAsset;

use yororen_ui_locale_en as locale_en;
use yororen_ui_theme_system as theme_system;

mod a11y_app;
mod state;

use state::ModalA11yState;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        yororen_ui::component::init(cx);
        theme_system::install(cx, cx.window_appearance());
        locale_en::install(cx);

        let st = ModalA11yState::new(cx);
        cx.set_global(st);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(720.0), px(440.0)),
                cx,
            ))),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| cx.new(a11y_app::ModalA11yApp::new));
    });
}
