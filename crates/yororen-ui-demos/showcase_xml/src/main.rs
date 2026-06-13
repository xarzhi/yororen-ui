//! yororen-ui Showcase (XML edition).
//!
//! Demonstrates the full Phase 1 + Phase 2 feature
//! surface of the `xml!` macro: containers, leaves,
//! event handlers, control flow (`If`/`ElseIf`/`Else`/
//! `For`), inline expressions, multi-arg factories
//! (via the `<Heading>` component), two-way binding
//! (`@bind`), conditional rendering, and string
//! interpolation.
//!
//! ## Run
//!
//! ```bash
//! cargo run -p showcase-xml-demo
//! ```

mod state;
mod view;

use gpui::{
    App, AppContext, Application, InteractiveElement, IntoElement, WindowBounds, WindowOptions,
    div, px, size,
};

use yororen_ui::assets::UiAsset;
use yororen_ui::locale_en;
use yororen_ui::renderer;

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

        // English translations.
        locale_en::install(cx);

        // Set up the showcase's app state.
        let showcase = state::ShowcaseState::new(cx);
        cx.set_global(showcase);

        // Open the main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(640.0), px(480.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(|_cx| view::ShowcaseApp));
    });
}
