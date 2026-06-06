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

use gpui::{Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window, div, px};
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use yororen_ui::renderer::DefaultButton;
use yororen_ui::renderer::DefaultLabel;
use yororen_ui::ActionVariantKind;

pub struct LayersApp;

impl LayersApp {
    pub fn new() -> Self {
        Self
    }
}

impl Render for LayersApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Column 1: pure headless — caller draws a black
        // square. The button's `.apply(div())` is just
        // plumbing for focus + click; the visible
        // hover/active feedback is `apply`'s built-in
        // opacity dip (light → 0.9 on hover, → 0.85 on
        // press), so even a bare caller `div()` shows
        // *some* interaction. Caller sets `text_color`
        // explicitly — without it, the text inherits the
        // gpui default (black) and disappears against a
        // black bg.
        let headless_btn = button("headless-only", &mut **cx)
            .on_click(|_, _, _| {})
            .apply(
                div()
                    .bg(gpui::hsla(0.0, 0.0, 0.05, 1.0))
                    .text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                    .p_2()
                    .rounded(px(4.))
                    .child("click me"),
            );

        // Column 2: headless + default-renderer sugar. Uses
        // the demo theme's `Neutral` action palette — pure
        // black `#0A0A0A`, hover `#2A2A2A`, active `#1A1A1A`
        // (modern monochrome, ~8% lightness delta on
        // hover).
        let default_btn = button("default-render", &mut **cx)
            .variant(ActionVariantKind::Neutral)
            .default_render(cx)
            .child("Click me");

        // Column 3: headless + caller fully custom. The
        // caller paints its own background, border, padding
        // and radius — and owns the hover/active styling too
        // (so it can pick a visibly-different transition;
        // the apply's built-in opacity dip is too subtle on
        // a white surface). `.raw_hover(false)` disables
        // `apply`'s default feedback; the caller chains
        // `.hover() / .active()` after `apply(el)`.
        let custom_btn = button("custom", &mut **cx)
            .variant(ActionVariantKind::Danger)
            .raw_hover(false)
            .apply(
                div()
                    .bg(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                    .border_1()
                    .border_color(gpui::hsla(0.0, 0.0, 0.1, 1.0))
                    .px(px(16.))
                    .py(px(8.))
                    .rounded(px(8.))
                    .text_color(gpui::hsla(0.0, 0.0, 0.05, 1.0))
                    .child("Click me"),
            )
            .hover(|s| {
                s.bg(gpui::hsla(0.0, 0.0, 0.92, 1.0))
                    .border_color(gpui::hsla(0.0, 0.0, 0.0, 1.0))
            })
            .active(|s| {
                s.bg(gpui::hsla(0.0, 0.0, 0.85, 1.0))
                    .border_color(gpui::hsla(0.0, 0.0, 0.0, 1.0))
            });

        div()
            .id("layers-scroll")
            .size_full()
            .bg(gpui::hsla(0.0, 0.0, 0.97, 1.0))
            .flex()
            .flex_col()
            .gap(px(24.))
            .p(px(24.))
            .overflow_y_scroll()
            .child(panel_body(
                "1. Headless only",
                "Caller writes every visual: bg, padding, radius, text. The button is just a focus + click handler.",
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
                "3. + Caller custom",
                "headless::button + caller-owned div: bg, border, padding, radius all written by the user. The renderer is bypassed; headless only wires a11y + click.",
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(custom_btn)
                    .child(label("caption", "custom caption", &mut **cx).default_render(cx)),
                cx,
            ))
            .child({
                // Bind inputs_strip to a local first so the
                // `&mut **cx` borrow is released before
                // `panel_body` re-borrows `cx` for its own
                // label/div wiring.
                let inputs = text_input_strip(window, cx);
                panel_body(
                    "4. Inputs (hover border)",
                    "TextInput: border defaults to `border.default`, hover → `border.muted`, press → `border.default` (deeper). Hover to see.",
                    inputs,
                    cx,
                )
            })
    }
}

fn text_input_strip(window: &mut Window, cx: &mut Context<LayersApp>) -> impl IntoElement + use<> {
    use yororen_ui::headless::text_input::text_input;
    use yororen_ui::renderer::DefaultTextInput;
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .child(
            text_input("demo-text-input")
                .placeholder("Type here…")
                .default_render(cx, window),
        )
}

fn panel_body(
    title: &str,
    blurb: &str,
    body: impl IntoElement,
    cx: &mut Context<LayersApp>,
) -> impl IntoElement {
    div()
        .w_full()
        .bg(gpui::hsla(0.0, 0.0, 1.0, 1.0))
        .rounded(px(8.))
        .p(px(16.))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            label("title", title, &mut **cx)
                .strong(true)
                .default_render(cx),
        )
        .child(
            label("blurb", blurb, &mut **cx)
                .wrap()
                .default_render(cx)
                .text_size(gpui::px(13.))
                .w_full(),
        )
        .child(body)
}
