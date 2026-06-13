//! yororen-ui Showcase (XML edition).
//!
//! Demonstrates the full Phase 1 + Phase 2 + Phase 3
//! feature surface of the `xml!` / `xml_file!` macro:
//! containers, leaves, event handlers, control flow
//! (`If` / `ElseIf` / `Else` / `For` / `Match` /
//! `Case`), inline expressions, two-way binding
//! (`@bind`), local state (`<State>`), event modifiers
//! (`.stop` / `.prevent` / keyboard filters), string
//! interpolation, and runtime-registered custom
//! components.
//!
//! ## Architecture
//!
//! ```
//! state.rs        — pure data: entities, enums, lists
//! controller.rs   — business logic; one method per
//!                   on_click / on_change handler
//! view.rs         — Render impl; reads state, hands
//!                   the controller to XML
//! ui/showcase.xml — purely declarative; references
//!                   controller methods by name
//! ```
//!
//! The XML file is the **single source of truth for
//! the interface**. Every business decision (counter
//! increments, status transitions, name updates) is
//! expressed in Rust — the XML never contains
//! `move |_, _, cx| { ... }` closures or `update(cx,
//! |s, _| ...)` boilerplate.
//!
//! ## Run
//!
//! ```bash
//! cargo run -p showcase-xml-demo
//! ```

mod controller;
mod state;
mod view;

use gpui::{
    App, AppContext, Application, InteractiveElement, IntoElement, WindowBounds, WindowOptions,
    div, px, size,
};

use yororen_ui::assets::UiAsset;
use yororen_ui::locale_en;
use yororen_ui::renderer;

use crate::controller::Controller;
use crate::state::{ShowcaseState, StateRef};
use crate::view::ShowcaseApp;

/// A trivial custom widget used to exercise
/// `register_xml_component!` — the user-facing extension
/// point for adding new XML tags at runtime.
fn render_custom_widget(id: &str, _cx: &mut gpui::App) -> gpui::AnyElement {
    div().id(id.to_string()).into_any_element()
}

yororen_ui::register_xml_component!(CustomWidget => render_custom_widget);

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Install the default theme + renderers.
        renderer::install(cx, cx.window_appearance());

        // Bind the global keymap for text-input
        // actions (Backspace, Delete, Left/Right,
        // SelectAll, Paste, etc.). Without this the
        // TextInput can't accept backspace or other
        // editing keys - they're bound to the
        // "UITextInput" key context that this
        // registers (idempotent — safe to call once).
        yororen_ui::headless::text_input::init(cx);

        // English translations.
        locale_en::install(cx);

        // Build the state and the controller that owns it.
        // The controller is `Clone` so each event handler
        // closure in the XML gets its own handle.
        let state = cx.new(|cx| ShowcaseState::new_data(cx));
        let controller = Controller::new(state.clone());

        // Make the state available to the view as a global
        // (the view reads it via `cx.global::<StateRef>()`).
        cx.set_global(StateRef { state });

        // Open the main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(720.0), px(620.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| {
            cx.new(|cx| ShowcaseApp::new(cx, controller))
        });
    });
}
