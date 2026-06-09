//! `BrutalismApp` — the single-window component the demo
//! renders. Builds a vertical stack of every component that
//! has a `DefaultXxx` sugar trait, so the user can eyeball the
//! style at a glance.
//!
//! Notes on the layout:
//!
//! - The root `div` sets `bg(surface.canvas)` so the window
//!   background is the light-theme off-white, not the
//!   macOS-dark-mode default.
//! - The headless `heading` factory's `apply()` does not
//!   render its `text` field, so the demo builds the H1 by
//!   hand with `font_weight(FontWeight(800)).text_3xl()`
//!   instead of going through the headless `heading`.
//! - `Switch` / `Checkbox` / `Radio` ship a fixed-width
//!   track inside `default_render`. Putting a label as a
//!   direct `.child(...)` of the rendered track clips the
//!   text. Each one is wrapped in a `flex()` row with the
//!   label as a sibling instead.
//!
//! Every interactive component on screen is a
//! `headless::Xxx` factory + `.render(cx)`. No custom
//! `div()` composition for the components themselves — the
//! renderer alone defines the visual vocabulary.

use gpui::{Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px};
use yororen_ui::ActionVariantKind;
use yororen_ui::headless::{
    button::button, checkbox::checkbox, icon_button::icon_button, label::label, radio::radio,
    switch::switch, toggle_button::toggle_button,
};
use yororen_ui::theme::ActiveTheme;

pub struct BrutalismApp;

impl BrutalismApp {
    pub fn new() -> Self {
        Self
    }
}

impl Render for BrutalismApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The root background. The brutalism-light theme defines
        // `surface.canvas` as `#F4F4F4`; we read it through the
        // theme so the demo works for both light and dark mode
        // if the user switches at the OS level.
        let canvas_bg = cx.theme().get_color("surface.canvas").unwrap_or_default();
        let panel_bg = cx.theme().get_color("surface.base").unwrap_or_default();
        let title_color = cx.theme().get_color("content.primary").unwrap_or_default();

        // Build children. Each headless factory + default_render
        // must run in its own block so the `&mut App` borrow
        // from `&mut **cx` is released before the next one.
        let mut children: Vec<gpui::AnyElement> = Vec::new();

        // --- Title (built by hand; see header note) ---
        children.push(
            div()
                .font_weight(FontWeight(800.0))
                .text_3xl()
                .text_color(title_color)
                .child("BRUTALISM")
                .into_any_element(),
        );
        children.push(
            label(
                "brutalism-subtitle",
                "Neo-brutalism — sharp corners, 3px black borders, hard offset shadows, monospace.",
                &mut **cx,
            )
            .render(cx)
            .into_any_element(),
        );

        // --- Buttons row (4 variants) ---
        children.push(
            div()
                .flex()
                .gap(px(12.0))
                .child(
                    button("btn-neutral", &mut **cx)
                        .variant(ActionVariantKind::Neutral)
                        .on_click(|_, _, _| {})
                        .render(cx)
                        .child("NEUTRAL"),
                )
                .child(
                    button("btn-primary", &mut **cx)
                        .variant(ActionVariantKind::Primary)
                        .on_click(|_, _, _| {})
                        .render(cx)
                        .child("PRIMARY"),
                )
                .child(
                    button("btn-danger", &mut **cx)
                        .variant(ActionVariantKind::Danger)
                        .on_click(|_, _, _| {})
                        .render(cx)
                        .child("DANGER"),
                )
                .child(
                    button("btn-disabled", &mut **cx)
                        .variant(ActionVariantKind::Primary)
                        .disabled(true)
                        .on_click(|_, _, _| {})
                        .render(cx)
                        .child("DISABLED"),
                )
                .into_any_element(),
        );

        // --- IconButton + ToggleButton row ---
        children.push(
            div()
                .flex()
                .gap(px(12.0))
                .items_center()
                .child(
                    icon_button("ibtn-1", &mut **cx)
                        .on_click(|_, _, _| {})
                        .render(cx)
                        .child("◆"),
                )
                .child(
                    toggle_button("tbtn-1", &mut **cx)
                        .selected(true)
                        .on_toggle(|_, _, _, _| {})
                        .render(cx)
                        .child("TOGGLED ON"),
                )
                .child(
                    toggle_button("tbtn-2", &mut **cx)
                        .selected(false)
                        .on_toggle(|_, _, _, _| {})
                        .render(cx)
                        .child("TOGGLED OFF"),
                )
                .into_any_element(),
        );

        // --- Form controls ---
        //
        // Each switch / checkbox / radio is a *fixed-width*
        // track inside `default_render`. To put a label next
        // to it without clipping, the demo wraps the rendered
        // component + label in a flex row. The track itself
        // stays exactly the size the renderer wants.
        let mut form_rows: Vec<gpui::AnyElement> = Vec::new();
        form_rows.push(
            div()
                .flex()
                .gap(px(8.0))
                .items_center()
                .child(
                    switch("sw-on", &mut **cx)
                        .checked(true)
                        .on_toggle(|_, _, _, _| {})
                        .render(cx),
                )
                .child(
                    label("lbl-sw-on", "Switch ON", &mut **cx)
                        .render(cx)
                        .into_any_element(),
                )
                .into_any_element(),
        );
        form_rows.push(
            div()
                .flex()
                .gap(px(8.0))
                .items_center()
                .child(
                    switch("sw-off", &mut **cx)
                        .checked(false)
                        .on_toggle(|_, _, _, _| {})
                        .render(cx),
                )
                .child(
                    label("lbl-sw-off", "Switch OFF", &mut **cx)
                        .render(cx)
                        .into_any_element(),
                )
                .into_any_element(),
        );
        form_rows.push(
            div()
                .flex()
                .gap(px(8.0))
                .items_center()
                .child(
                    checkbox("cb-1", &mut **cx)
                        .checked(true)
                        .on_toggle(|_, _, _, _| {})
                        .render(cx),
                )
                .child(
                    label("lbl-cb-1", "Checkbox", &mut **cx)
                        .render(cx)
                        .into_any_element(),
                )
                .into_any_element(),
        );
        form_rows.push(
            div()
                .flex()
                .gap(px(8.0))
                .items_center()
                .child(
                    radio("rd-1", &mut **cx)
                        .checked(true)
                        .on_toggle(|_, _, _, _| {})
                        .render(cx),
                )
                .child(
                    label("lbl-rd-1", "Radio A", &mut **cx)
                        .render(cx)
                        .into_any_element(),
                )
                .into_any_element(),
        );
        children.push(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .children(form_rows)
                .into_any_element(),
        );

        // --- TextInput ---
        // (text_input not yet supported in headless render())

        // Compose. Root div sets the canvas background so the
        // window isn't transparent on macOS dark mode.
        div().flex().flex_col().p(px(24.0)).bg(canvas_bg).child(
            // Inner "card-like" panel so the components sit
            // on top of a slightly lighter base. This is
            // the neo-brutalism "panel" aesthetic: a
            // white-on-cream stack rather than a fully
            // flat single colour.
            div()
                .flex()
                .flex_col()
                .gap(px(20.0))
                .p(px(24.0))
                .bg(panel_bg)
                .children(children),
        )
    }
}
