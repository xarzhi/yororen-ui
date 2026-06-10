//! yororen-ui Layers Demo
//!
//! Three side-by-side panels showing the v0.3 three-layer
//! architecture in action:
//!
//! 1. **Headless only** — every visual decision is the
//!    caller's; the `headless::button` returns a `ButtonProps`
//!    that the caller composes with a raw `div()`.
//! 2. **Headless + default-renderer** — same `headless::button`,
//!    but `.render(cx)` reads the registered
//!    `TokenButtonRenderer` and applies the default look.
//! 3. **Headless + caller custom (Material ripple)** — same
//!    `headless::button` factory, but the caller paints a
//!    Material-Design-style teal background and a custom
//!    `RippleElement` paints an opaque circle that expands from
//!    each click point while its color lerps back to the teal
//!    background (the circle "dissolves" into the button over
//!    450ms). Driven by `window.request_animation_frame`.

use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, Window, div, px,
};
use yororen_ui::ActionVariantKind;
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;

use crate::material_button::material_button;

pub struct LayersApp;

impl LayersApp {
    pub fn new() -> Self {
        Self
    }
}

impl Render for LayersApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Column 1: pure headless — caller writes every pixel.
        // `apply` is purely a11y: focus + click. There is no
        // built-in hover / active feedback. The only
        // interactive signal is the cursor (we set it to
        // PointingHand explicitly so it's visible). To get
        // hover feedback, the caller must add their own
        // `.hover(...).active(...)` — see panel 3.
        let headless_btn = button("headless-only", cx).on_click(|_, _, _| {}).apply(
            div()
                .bg(gpui::hsla(0.0, 0.0, 0.05, 1.0))
                .text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                .p_2()
                .rounded(px(4.))
                .cursor(gpui::CursorStyle::PointingHand)
                .child("click me"),
        );

        // Column 2: headless + default-renderer sugar. Uses
        // the demo theme's `Neutral` action palette — pure
        // black `#0A0A0A`, hover `#2A2A2A`, active `#1A1A1A`
        // (modern monochrome, ~8% lightness delta on
        // hover).
        let default_btn = button("default-render", cx)
            .variant(ActionVariantKind::Neutral)
            .render(cx)
            .child("Click me");

        // Column 3: headless + caller fully custom. The caller
        // paints its own Material-Design-style background, radius,
        // typography, and the click ripple animation. The
        // ripple is a custom `gpui::Element` that uses
        // `PathBuilder` to construct a circular path and
        // `window.paint_path` to draw it as a true circle (not
        // a square — `paint_quad` would fill a square, which
        // shows up as a block of color at full radius). The
        // circle starts small at the click point, grows to
        // cover the button over 450ms (ease-out cubic), and
        // its alpha fades linearly from 0.20 to 0. The
        // button's `overflow: hidden` clips each ripple to
        // the rounded shape. `material_button` is the demo's
        // bespoke headless-button renderer; the headless
        // layer (focus + click) is still provided by `apply`.
        let custom_btn = material_button("material-custom", "Click me".into(), &mut **cx, window);

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
                "1. Headless only (no built-in feedback)",
                "`headless::button` only wires a11y: focus + click. The button does **not** visually respond to hover or press — try hovering, nothing changes (only the cursor becomes a pointer). Visual feedback is the caller's responsibility; see panel 3 for the caller-painted version.",
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(headless_btn)
                    .child(label("caption", "headless caption", cx).render(cx)),
                cx,
            ))
            .child(panel_body(
                "2. + Default renderer",
                "headless::button + .render(cx) uses the installed TokenButtonRenderer. Padding, radius, bg all come from the JSON theme.",
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(default_btn)
                    .child(label("caption", "default caption", cx).render(cx)),
                cx,
            ))
            .child(panel_body(
                "3. + Caller custom (Material Design + ripple)",
                "Same headless factory as panel 1, but the caller paints the entire look: M2 raised-button teal background, 4px radius, 14px medium-weight label, hover lightens the fill. On click, a 20%-alpha white circle expands from the click point over 450ms (ease-out cubic) and fades to 0 alpha — the classic Material ripple. The ripple is a custom `gpui::Element` (built with `PathBuilder` so the painted shape is a true circle, not a square) and the animation is driven by `window.request_animation_frame`; try clicking different parts of the button to see each ripple emanate from that point.",
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(custom_btn)
                    .child(label("caption", "material caption", cx).render(cx)),
                cx,
            ))
            .child({
                // Bind inputs_strip to a local first so the
                // `&mut **cx` borrow is released before
                // `panel_body` re-borrows `cx` for its own
                // label/div wiring.
                let inputs = text_input_strip(window, cx);
                panel_body(
                    "4. Default renderer also covers inputs",
                    "Panels 1–3 prove the headless / renderer split for `button`. This panel proves it works the same for `text_input`: the headless factory is the same one inputs_demo uses, and `.render(cx, window)` reads `TokenTextInputRenderer` for bg / border / padding / focus styling. Hover to see `border.default` → `border.muted`, click to focus (border deepens to `border.focus`).",
                    inputs,
                    cx,
                )
            })
    }
}

fn text_input_strip(window: &mut Window, cx: &mut Context<LayersApp>) -> impl IntoElement + use<> {
    use yororen_ui::headless::text_input::text_input;
    div().flex().flex_col().gap_2().w_full().child(
        text_input("demo-text-input")
            .placeholder("Type here…")
            .render(cx, window),
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
        .child(label("title", title, cx).strong(true).render(cx))
        .child(
            label("blurb", blurb, cx)
                .wrap()
                .render(cx)
                .text_size(gpui::px(13.))
                .w_full(),
        )
        .child(body)
}
