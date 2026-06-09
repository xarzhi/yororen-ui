//! yororen-ui Mini Renderer Demo
//!
//! Mini-styled component gallery. Every button in this
//! window is rendered by `yororen-ui-mini-renderer`, which
//! only reads `themeColor` + `accentColor` from the theme
//! JSON. Padding, radius, height are baked into code.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use yororen_ui_default_renderer::DefaultButton;
use yororen_ui_default_renderer::DefaultLabel;

pub struct MiniApp {
    counter: i32,
}

impl MiniApp {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl Render for MiniApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Each `.default_render(cx)` resolves to the
        // `MiniButtonRenderer` because it was installed last
        // (overwrites the default registration in the core
        // `RendererRegistry`).
        let _inc = button("inc", cx).default_render(cx);
        let _reset = button("reset", cx).default_render(cx);

        div()
            .size_full()
            .p(px(24.))
            .flex()
            .flex_col()
            .gap_3()
            .child(label("title", "Mini renderer demo", cx).default_render(cx))
            .child(
                label(
                    "blurb",
                    "Install order: default-renderer first, then mini. Last registration wins. The mini only overrides Button / IconButton / ToggleButton / Label.",
                    cx,
                )
                .default_render(cx),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(label("c", format!("Counter: {}", self.counter), cx).default_render(cx))
                    .child(button("inc-btn", cx).default_render(cx))
                    .child(button("reset-btn", cx).default_render(cx)),
            )
    }
}
