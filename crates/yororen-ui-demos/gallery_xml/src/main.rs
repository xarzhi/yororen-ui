//! Gallery Demo (XML edition) — minimal port of the
//! `gallery_demo` binary that drives the same UI but
//! describes its layout entirely in XML. The state
//! and event-handler layout mirror `showcase_xml`'s
//! controller pattern: data lives in pure structs
//! wrapped in entities, business logic in a `Controller`
//! the XML references by method name.
//!
//! Compared to the full `gallery_demo` we make two
//! pragmatic simplifications to keep the XML readable:
//!
//! 1. **No renderer switcher** — the default renderer is
//!    installed once at startup. Hot-swapping renderers
//!    per render is a Rust-side concern that the XML
//!    layer doesn't need to express.
//! 2. **No locale switcher** — the English locale is
//!    installed once; section titles / cell labels use
//!    plain string literals.
//!
//! Everything else — the toolbar, the 7 sections, the
//! inputs, the controls, the lists — is XML. The
//! `controller` field is `Clone` so the auto-wrap in
//! `<Button on_click={controller.foo}>` captures one
//! clone per handler.
//!
//! ## Run
//!
//! ```bash
//! cargo run -p gallery-xml-demo
//! ```

mod controller;
mod state;
mod view;

use gpui::{
    App, AppContext, Application, InteractiveElement, IntoElement, WindowBounds, WindowOptions,
    div, px, size,
};

use yororen_ui::assets::UiAsset;
use yororen_ui::notification::center::NotificationCenter;
use yororen_ui::renderer;

use crate::controller::Controller;
use crate::state::{GalleryState, StateRef};
use crate::view::GalleryApp;

/// Trivial custom widget used to exercise
/// `register_xml_component!` — the extension hook for
/// adding tags without touching the codegen schema.
fn render_counter_widget(id: &str, _cx: &mut gpui::App) -> gpui::AnyElement {
    div().id(id.to_string()).into_any_element()
}

yororen_ui::register_xml_component!(CounterWidget => render_counter_widget);

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // 1. Install the default theme + renderers (light).
        renderer::install(cx, cx.window_appearance());

        // 2. Bind the text-input keymap once (idempotent).
        //    Without this, the TextInput can't accept
        //    backspace or other editing keys.
        yororen_ui::headless::text_input::init(cx);

        // 3. Install the notification center (toast /
        //    notification trigger from the toolbar).
        cx.set_global(NotificationCenter::new());

        // 4. Build the state + the controller that owns
        //    it. The controller is `Clone` so the XML's
        //    event handlers each get their own handle.
        let state = cx.new(|cx| GalleryState::new_data(cx));
        let controller = Controller::new(state.clone());

        // 5. Make the state available to the view as a
        //    global (read via `cx.global::<StateRef>()`).
        cx.set_global(StateRef { state });

        // 6. Open the main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(1100.0), px(820.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| {
            cx.new(|cx| GalleryApp::new(cx, controller))
        });
    });
}
