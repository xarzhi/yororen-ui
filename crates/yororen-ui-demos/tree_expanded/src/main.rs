//! Tree Expanded State Demo
//!
//! This demo verifies that `collect_expanded` does not overwrite user-managed
//! expansion state when nodes are rebuilt.
//!
//! ## What to check
//! - Expand/collapse nodes via clicking
//! - The state panel updates to reflect the current expanded state
//! - "Reset with new nodes" preserves the user's expansion choices
//! - "Force re-render" does not reset expansion state
//!
//! ## Run
//! ```bash
//! cd demo/tree_expanded && cargo run
//! ```

mod state;
mod tree_app;

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

        let tree_state = state::TreeDemoState::new(cx);
        cx.set_global(tree_state);

        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(560.0), px(480.0)),
                cx,
            ))),
            ..Default::default()
        };

        cx.open_window(options, |_, cx| {
            cx.new(|cx| tree_app::TreeExpandedApp::new(cx))
        })
        .unwrap();
    });
}
