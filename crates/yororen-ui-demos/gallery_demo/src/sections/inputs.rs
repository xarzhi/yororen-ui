//! Section 4 — Inputs. Each component instance is wrapped in a
//! labelled `cell` so the user can identify every input.

use gpui::{Context, Div, IntoElement, ParentElement, Styled, Window, div, px};

use yororen_ui::headless::combo_box::combo_box;
use yororen_ui::headless::file_path_input::file_path_input;
use yororen_ui::headless::keybinding_input::KeybindingInputMode;
use yororen_ui::headless::keybinding_input::keybinding_input;
use yororen_ui::headless::label::label;
use yororen_ui::headless::number_input::number_input;
use yororen_ui::headless::password_input::password_input;
use yororen_ui::headless::search_input::search_input;
use yororen_ui::headless::select::select;
use yororen_ui::headless::text_area::text_area;
use yororen_ui::headless::text_input::text_input;
use yororen_ui::i18n::Translate;

use crate::state::GalleryApp;

pub fn render(
    app: &mut GalleryApp,
    window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> Div {
    let entity = cx.entity().clone();
    let value_prefix = cx.t("demo.input.value").to_string();
    let mode_prefix = cx.t("demo.input.mode").to_string();

    // text_input
    let entity_text = entity.clone();
    let text_value = app.text_value.clone();
    let text_input_el = text_input("input-text")
        .placeholder(cx.t("demo.input.placeholder_type_here"))
        .on_change(move |new: &str, _w, cx| {
            entity_text.update(cx, |s, _cx| s.text_value = new.to_string());
        })
        .render(cx, window);

    // password_input
    let entity_pw = entity.clone();
    let pw_value = app.password_value.clone();
    let pw_input_el = password_input("input-password")
        .placeholder(cx.t("demo.input.placeholder_password"))
        .mask_char('•')
        .on_change(move |new: &str, _w, cx| {
            entity_pw.update(cx, |s, _cx| s.password_value = new.to_string());
        })
        .render(cx, window);

    // number_input
    let entity_num = entity.clone();
    let entity_num_inc = entity.clone();
    let entity_num_dec = entity.clone();
    let num_value = app.number_value;
    let num_input_el = number_input("input-number")
        .min(0.0)
        .max(100.0)
        .step(1.0)
        .value(num_value)
        .on_change(move |new: f64, _w, cx| {
            entity_num.update(cx, |s, _cx| s.number_value = new);
        })
        .on_increment(move |next: f64, _w, cx| {
            entity_num_inc.update(cx, |s, _cx| s.number_value = next);
        })
        .on_decrement(move |next: f64, _w, cx| {
            entity_num_dec.update(cx, |s, _cx| s.number_value = next);
        })
        .render(cx, window);

    // search_input
    let entity_search = entity.clone();
    let search_value = app.search_value.clone();
    let search_input_el = search_input("input-search")
        .placeholder(cx.t("demo.input.placeholder_search"))
        .on_change(move |new: &str, _w, cx| {
            entity_search.update(cx, |s, _cx| s.search_value = new.to_string());
        })
        .on_clear(|_w, _cx| {})
        .render(cx, window);

    // file_path_input
    let entity_fp = entity.clone();
    let fp_value = app.file_path_value.clone();
    let fp_input_el = file_path_input("input-file-path")
        .placeholder(cx.t("demo.input.placeholder_path"))
        .on_change(move |new: &str, _w, cx| {
            entity_fp.update(cx, |s, _cx| s.file_path_value = new.to_string());
        })
        .on_browse(|_picked, _w, _cx| {})
        .render(cx, window);

    // keybinding_input
    let entity_kb = entity.clone();
    let entity_kb_start = entity.clone();
    let entity_kb_cancel = entity.clone();
    let kb_value = app.keybinding_value.clone();
    let kb_mode = app.keybinding_mode;
    let kb_input_el = keybinding_input("input-keybinding")
        .mode(kb_mode)
        .on_change(move |new: &str, _w, cx| {
            entity_kb.update(cx, |s, _cx| s.keybinding_value = new.to_string());
        })
        .on_start_capture(move |_w, cx| {
            entity_kb_start.update(cx, |s, _cx| {
                s.keybinding_mode = KeybindingInputMode::Capturing;
            });
        })
        .on_cancel_capture(move |_w, cx| {
            entity_kb_cancel.update(cx, |s, _cx| {
                s.keybinding_mode = KeybindingInputMode::Idle;
            });
        })
        .render(cx, window);

    // text_area
    let entity_ta = entity.clone();
    let ta_value = app.text_area_value.clone();
    let ta_input_el = text_area("input-text-area")
        .placeholder(cx.t("demo.input.placeholder_text_area"))
        .on_change(move |new: &str, _w, cx| {
            entity_ta.update(cx, |s, _cx| s.text_area_value = new.to_string());
        })
        .render(cx, window);

    // composite option lists
    let entity_select = entity.clone();
    let select_state = app.select_state.clone();
    select_state.update(cx, |s, _cx| {
        s.set_on_change(move |value, _w, cx| {
            let v = value.to_string();
            entity_select.update(cx, |s, _cx| {
                s.select_demo_value = v;
            });
        });
    });
    let select_el = select("input-select", select_state.clone()).render(cx);

    let entity_combo = entity.clone();
    let combo_state = app.combo_state.clone();
    combo_state.update(cx, |s, _cx| {
        s.set_on_change(move |value, _w, cx| {
            let v = value.to_string();
            entity_combo.update(cx, |s, _cx| {
                s.combo_demo_value = v;
            });
        });
    });
    let combo_el = combo_box("input-combo", combo_state.clone()).render(cx, window);

    // assemble — each input goes in its own labelled cell,
    // followed by a status line that shows the live value.
    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(input_cell(cx.t("input.text"), text_input_el, &format!("{value_prefix} {:?}", text_value), cx))
        .child(input_cell(cx.t("input.password"), pw_input_el, &format!("{value_prefix} {:?}", pw_value), cx))
        .child(input_cell(cx.t("input.number"), num_input_el, &format!("{value_prefix} {}", num_value), cx))
        .child(input_cell(cx.t("input.search"), search_input_el, &format!("{value_prefix} {:?}", search_value), cx))
        .child(input_cell(cx.t("demo.input.file_path_hint"), fp_input_el, &format!("{value_prefix} {:?}", fp_value), cx))
        .child(input_cell(cx.t("demo.input.keybinding_hint"), kb_input_el, &format!("{value_prefix} {:?}  {mode_prefix} {:?}", kb_value, kb_mode), cx))
        .child(input_cell(cx.t("demo.input.text_area_hint"), ta_input_el, &format!("{value_prefix} {:?}", ta_value), cx))
        .child(input_cell(cx.t("input.select"), select_el, &format!("{value_prefix} {}", app.select_demo_value), cx))
        .child(input_cell(cx.t("input.combo"), combo_el, &format!("{value_prefix} {}", app.combo_demo_value), cx))
}

/// Render a labelled input cell with a status line below it
/// that shows the live value. The component name is the first
/// line, the input is the second, and the status is the third.
/// Reused by `controls.rs` so checkbox/switch can show their
/// `value: <bool>` status the same way the other inputs do.
pub fn input_cell(
    name: impl Into<String>,
    el: impl IntoElement,
    status: &str,
    cx: &mut Context<GalleryApp>,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .p(px(8.))
        .rounded(px(6.))
        .border_1()
        .border_color(gpui::hsla(0.0, 0.0, 0.5, 0.15))
        .child(
            label("input-name", name, cx)
                .muted(true)
                .render(cx)
                .text_size(px(11.)),
        )
        .child(el)
        .child(
            label("input-status", status, cx)
                .muted(true)
                .render(cx)
                .text_size(px(11.)),
        )
}

