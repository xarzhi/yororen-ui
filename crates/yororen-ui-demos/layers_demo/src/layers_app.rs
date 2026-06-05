//! yororen-ui Layers Demo
//!
//! Three side-by-side panels showing the v0.3 three-layer
//! architecture in action:
//!
//! 1. **Headless only** — every visual decision is the
//!    caller's; the `headless::button` returns a `ButtonProps`
//!    that the caller composes with a raw `div()`.
//! 2. **Headless + default-renderer** — same `headless::button`,
//!    but `.default_render(cx)` reads the registered
//!    `TokenButtonRenderer` and applies the default look.
//! 3. **Headless + mini-renderer override** — same `headless::button`,
//!    but a custom `MiniButtonRenderer` is installed on top
//!    of the default; the button picks up the mini's `themeColor`
//!    while the surrounding label / div still come from
//!    default-renderer.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use yororen_ui::renderer::DefaultButton;
use yororen_ui::renderer::DefaultLabel;

pub struct LayersApp;

impl LayersApp {
    pub fn new() -> Self {
        Self
    }
}

impl Render for LayersApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Column 1: pure headless — caller draws a red
        // square. No renderer is read at all; the button's
        // `.apply(div())` is just plumbing.
        let headless_btn = button("headless-only", &mut **cx)
            .on_click(|_, _, _| {})
            .apply(div().bg(gpui::hsla(0.0, 0.6, 0.5, 1.0)).p_2().rounded(px(4.)).child("click me"));

        // Column 2: headless + default-renderer sugar.
        let default_btn = button("default-render", &mut **cx)
            .default_render(cx);

        // Column 3: headless + default-renderer + mini
        // override. The `default_render` call now resolves
        // to `MiniButtonRenderer` because it was installed
        // last and overwrites the same `markers::Button` key.
        let mini_btn = button("mini-override", &mut **cx)
            .default_render(cx);

        div()
            .size_full()
            .bg(gpui::hsla(0.0, 0.0, 0.97, 1.0))
            .flex()
            .flex_row()
            .gap(px(24.))
            .p(px(24.))
            .child(panel_body(
                "1. Headless only",
                "The caller writes every visual: bg, padding, radius, text. The button is just a focus + click handler.",
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(headless_btn)
                    .child(label("caption", "headless caption", &mut **cx).default_render(cx)),
                cx,
            ))
            .child(panel_body(
                "2. + Default renderer",
                "headless::button + .default_render(cx) uses the installed TokenButtonRenderer. Padding, radius, bg all come from the JSON theme.",
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(default_btn)
                    .child(label("caption", "default caption", &mut **cx).default_render(cx)),
                cx,
            ))
            .child(panel_body(
                "3. + Mini override",
                "Same call as column 2, but a MiniButtonRenderer was registered after the default. The mini wins because it was last to register.",
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(mini_btn)
                    .child(label("caption", "mini caption", &mut **cx).default_render(cx)),
                cx,
            ))
    }
}

fn panel_body(
    title: &str,
    blurb: &str,
    body: impl IntoElement,
    cx: &mut Context<LayersApp>,
) -> impl IntoElement {
    div()
        .flex_1()
        .bg(gpui::hsla(0.0, 0.0, 1.0, 1.0))
        .rounded(px(8.))
        .p(px(16.))
        .flex()
        .flex_col()
        .gap_2()
        .child(label("title", title, &mut **cx).default_render(cx))
        .child(label("blurb", blurb, &mut **cx).default_render(cx))
        .child(body)
}
