//! Section 1 — Actions.
//!
//! Each component is wrapped in a `cell` helper (defined in
//! `sections/mod.rs`) that shows a small `name` label above the
//! component itself so the user can identify what they're
//! looking at.
//!
//! Buttons / icon_buttons / toggle_buttons all accept a
//! `caption(...)` / `icon(...)` builder method so the demo
//! doesn't have to chain `.child(...)` after `.render(...)`.
//! Icon colour is derived from the renderer's `fg` token
//! automatically — no need to pass a hardcoded colour.

use gpui::{Context, Div, ParentElement, Styled, div, px};

use yororen_ui::ActionVariantKind;
use yororen_ui::headless::button::button;
use yororen_ui::headless::button_group::button_group;
use yororen_ui::headless::icon::IconSource;
use yororen_ui::headless::icon_button::icon_button;
use yororen_ui::headless::label::label;
use yororen_ui::headless::split_button::split_button;
use yororen_ui::headless::toggle_button::toggle_button;

use crate::sections::cell;
use crate::state::GalleryApp;

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Div {
    let entity = cx.entity().clone();

    // --- 3 button variants + disabled ---
    let row_buttons = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(12.))
        .child(cell("button / Neutral", button("btn-neutral", cx).variant(ActionVariantKind::Neutral).caption("Neutral").on_click(|_, _, _| {}).render(cx), cx))
        .child(cell("button / Primary", button("btn-primary", cx).variant(ActionVariantKind::Primary).caption("Primary").on_click(|_, _, _| {}).render(cx), cx))
        .child(cell("button / Danger", button("btn-danger", cx).variant(ActionVariantKind::Danger).caption("Danger").on_click(|_, _, _| {}).render(cx), cx))
        .child(cell("button / Disabled", button("btn-disabled", cx).disabled(true).caption("Disabled").on_click(|_, _, _| {}).render(cx), cx));

    // --- icon_button: variant + icon only, colour is
    //     auto-derived from the renderer's `fg` token. ---
    let row_icon_button = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(12.))
        .child(cell("icon_button (check)", icon_button("icon-btn-check", cx).on_click(|_, _, _| {}).icon(IconSource::Builtin("check".into())).render(cx), cx))
        .child(cell("icon_button / Primary (check)", icon_button("icon-btn-primary-check", cx).variant(ActionVariantKind::Primary).on_click(|_, _, _| {}).icon(IconSource::Builtin("check".into())).render(cx), cx));

    // --- toggle_button ---
    let entity_for_tb = entity.clone();
    let row_toggle = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .child(cell("toggle_button", toggle_button("toggle-1", cx).selected(app.toggle_btn_selected).caption("Press me").on_toggle(move |_selected, _ev, _window, cx| { entity_for_tb.update(cx, |s, _cx| { s.toggle_btn_selected = !s.toggle_btn_selected; }); }).render(cx), cx));

    // --- split_button (primary action + secondary on chevron) ---
    //     split_button is `apply()`-only — it has a custom two-slot
    //     layout (primary label + chevron label) that doesn't fit
    //     the simple caption/icon builder. We keep the explicit
    //     .child(...) chain here.
    let entity_for_split = entity.clone();
    let split = split_button("split-1", move |_ev, _w, cx| { entity_for_split.update(cx, |s, _cx| { s.toast_count += 1; }); }, cx)
        .on_secondary(|_ev, _w, _cx| {})
        .apply(div().flex().flex_row().items_center().gap(px(2.)))
        .child(div().px(px(12.)).py(px(6.)).child(label("split-label", "Save", cx).strong(true).render(cx)))
        .child(div().px(px(8.)).py(px(6.)).border_l_1().border_color(gpui::hsla(0.0, 0.0, 0.0, 0.2)).child(label("split-arrow", "▾", cx).render(cx)));
    let row_split = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .child(cell("split_button", split, cx));

    // --- button_group ---
    let row_group = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .child(cell("button_group (3 buttons)", button_group("btn-group-1", cx)
            .apply(div().flex().flex_row().rounded(px(6.)).overflow_hidden().border_1())
            .child(button("bg-left", cx).variant(ActionVariantKind::Neutral).on_click(|_, _, _| {}).apply(div().px(px(12.)).py(px(6.))).child("Left"))
            .child(button("bg-mid", cx).variant(ActionVariantKind::Neutral).on_click(|_, _, _| {}).apply(div().px(px(12.)).py(px(6.)).border_l_1().border_r_1()).child("Mid"))
            .child(button("bg-right", cx).variant(ActionVariantKind::Neutral).on_click(|_, _, _| {}).apply(div().px(px(12.)).py(px(6.))).child("Right")), cx));

    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(row_buttons)
        .child(row_icon_button)
        .child(row_toggle)
        .child(row_split)
        .child(row_group)
}
