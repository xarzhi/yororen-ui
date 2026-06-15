//! Section 5 — Controls. Each component is wrapped in a labelled
//! `cell` so the user can identify every instance.

use gpui::{Context, Div, ParentElement, Styled, div, px};

use yororen_ui::headless::checkbox::checkbox;
use yororen_ui::headless::label::label;
use yororen_ui::headless::radio::radio;
use yororen_ui::headless::radio_group::radio_group;
use yororen_ui::headless::slider::slider;
use yororen_ui::headless::switch::switch;
use yororen_ui::i18n::Translate;

use crate::sections::cell;
use crate::sections::input_cell;
use crate::state::GalleryApp;

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Div {
    let entity = cx.entity().clone();
    let value_prefix = cx.t("demo.input.value").to_string();

    // --- checkbox ---
    let entity_cb = entity.clone();
    let cb = checkbox("cbx-main", cx)
        .checked(app.checkbox_value)
        .on_toggle(move |v, _ev, _w, cx| {
            entity_cb.update(cx, |s, _cx| s.checkbox_value = v);
        })
        .render(cx);

    // --- switch ---
    let entity_sw = entity.clone();
    let sw = switch("swt-main", cx)
        .checked(app.switch_value)
        .on_toggle(move |v, _ev, _w, cx| {
            entity_sw.update(cx, |s, _cx| s.switch_value = v);
        })
        .render(cx);

    // --- 3 radio buttons in a radio_group ---
    let selected_label = cx.t("demo.controls.selected").to_string();
    let rg_label = label(
        "rg-current",
        format!("{selected_label} {}", app.radio_value),
        cx,
    )
    .muted(true)
    .render(cx);
    let rg_with_label = radio_group("rdg-choice", cx)
        .name("choice")
        .selected(app.radio_value)
        .apply(div().flex().flex_row().gap(px(8.)).items_center())
        .child(rg_label);
    let rg_with_radios = (0..3).fold(rg_with_label, |acc, i| {
        let entity_r = entity.clone();
        acc.child(
            radio(format!("rdo-{i}"), cx)
                .checked(app.radio_value == i)
                .on_toggle(move |_v, _ev, _w, cx| {
                    entity_r.update(cx, |s, _cx| s.radio_value = i);
                })
                .render(cx),
        )
    });

    // --- slider (unified renderer) ---
    let entity_sl = entity.clone();
    let slider_value = app.slider_value;
    let slider_track = slider("sld-value", cx)
        .value(slider_value)
        .range(0.0, 100.0)
        .step(1.0)
        .on_change(move |v, _w, cx| {
            entity_sl.update(cx, |s, _cx| s.slider_value = v);
        })
        .render(cx);

    let slider_status_template = cx.t("demo.controls.slider_value").to_string();
    let formatted_slider = format!("{:.1}", slider_value);
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(px(12.))
                .items_center()
                .child(input_cell(
                    cx.t("demo.controls.cell_checkbox"),
                    cb,
                    &format!("{value_prefix} {}", app.checkbox_value),
                    cx,
                ))
                .child(input_cell(
                    cx.t("demo.controls.cell_switch"),
                    sw,
                    &format!("{value_prefix} {}", app.switch_value),
                    cx,
                )),
        )
        .child(cell(
            cx.t("demo.controls.cell_radio_group"),
            rg_with_radios,
            cx,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(cell(cx.t("demo.controls.cell_slider"), slider_track, cx))
                .child(
                    label(
                        "slider-lbl",
                        slider_status_template.replacen("{}", &formatted_slider, 1),
                        cx,
                    )
                    .muted(true)
                    .render(cx),
                ),
        )
}
