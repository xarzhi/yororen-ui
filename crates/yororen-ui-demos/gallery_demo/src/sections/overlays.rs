//! Section 6 — Overlays. Each component is wrapped in a
//! labelled `cell`.

use gpui::{Context, Div, ParentElement, Styled, div, hsla, px};

use yororen_ui::headless::button::button;
use yororen_ui::headless::disclosure::disclosure;
use yororen_ui::headless::dropdown_menu::dropdown_menu;
use yororen_ui::headless::label::label;
use yororen_ui::headless::menu::menu;
use yororen_ui::headless::modal::modal;
use yororen_ui::headless::overlay::overlay;
use yororen_ui::headless::popover::popover;
use yororen_ui::headless::tooltip::tooltip;
use yororen_ui::i18n::Translate;

use crate::sections::cell;
use crate::state::GalleryApp;

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Div {
    // --- modal: trigger + scrim + body ---
    let modal_state_for_btn = app.modal_state.clone();
    let _open_modal_btn = button("ov-modal-open", cx)
        .on_click(move |_, _, cx| {
            modal_state_for_btn.update(cx, |st, _cx| st.open());
        })
        .render(cx)
        .child("Open modal");
    let modal_state = app.modal_state.clone();
    let is_modal_open = modal_state.read(cx).open;
    let modal_panel = modal("ov-modal", modal_state.clone())
        .render(cx)
        .w(px(360.))
        .child(label("ov-modal-title", cx.t("modal.title"), cx).strong(true).render(cx))
        .child(label("ov-modal-body", cx.t("modal.body"), cx).render(cx))
        .child(
            button("ov-modal-close", cx)
                .on_click(move |_, _, cx| {
                    modal_state.update(cx, |st, _cx| st.close());
                })
                .render(cx)
                .child("Close"),
        );
    let modal_body = if is_modal_open {
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 0.5))
            .child(modal_panel)
    } else {
        div()
    };
    let modal_wrapped = cell("modal (click to open)", modal_body, cx);

    // --- popover: trigger + popover body ---
    let popover_state_for_btn = app.popover_state.clone();
    let popover_state = app.popover_state.clone();
    let popover_is_open = popover_state.read(cx).open;
    let popover_trigger = button("ov-popover-trigger", cx)
        .on_click(move |_, _, cx| {
            popover_state_for_btn.update(cx, |st, _cx| st.toggle());
        })
        .render(cx)
        .child(if popover_is_open { "Close popover" } else { "Open popover" });
    let popover_body = if popover_is_open {
        let entity_for_menu = cx.entity().clone();
        let menu_state = app.menu_state.clone();
        menu_state.update(cx, |s, _cx| {
            s.set_on_select(move |id, _w, cx| {
                let id_s = id.to_string();
                entity_for_menu.update(cx, |s, _cx| {
                    s.menu_demo_value = id_s;
                });
            });
        });
        let menu_el = menu("ov-menu", menu_state.clone())
            .render(cx)
            .w(px(160.))
            .child(label("menu-blank", "", cx).render(cx));
        div().child(menu_el)
    } else {
        div()
    };
    let popover_el = popover("ov-popover", popover_state.clone())
        .render(cx)
        .child(popover_trigger)
        .child(popover_body);
    let popover_wrapped = cell("popover (click trigger)", popover_el, cx);

    // --- tooltip ---
    let tooltip_state = app.tooltip_state.clone();
    let tooltip_el = tooltip("ov-tooltip", cx.t("tooltip.text"), tooltip_state.clone())
        .render(cx)
        .child(label("ov-tt-target", "Hover or focus me", cx).render(cx));
    let tooltip_wrapped = cell("tooltip (hover target)", tooltip_el, cx);

    // --- dropdown_menu ---
    let entity_for_dropdown = cx.entity().clone();
    let dropdown_state = app.dropdown_state.clone();
    dropdown_state.update(cx, |s, _cx| {
        s.set_on_select(move |id, _w, cx| {
            let id_s = id.to_string();
            entity_for_dropdown.update(cx, |s, _cx| {
                s.dropdown_demo_value = id_s;
            });
        });
    });
    let dropdown_state_for_btn = dropdown_state.clone();
    let dropdown_trigger = button("ov-dropdown-trigger", cx)
        .on_click(move |_, _, cx| {
            dropdown_state_for_btn.update(cx, |st, _cx| st.toggle());
        })
        .render(cx)
        .child("Open menu ▾");
    let dropdown_body = if dropdown_state.read(cx).open {
        div()
            .w(px(160.))
            .p(px(4.))
            .rounded(px(6.))
            .border_1()
            .bg(gpui::rgb(0xFFFFFF))
            .child(label("dd-blank", "", cx).render(cx))
    } else {
        div()
    };
    let dropdown_el = dropdown_menu("ov-dropdown", dropdown_state.clone())
        .render(cx)
        .child(dropdown_trigger)
        .child(dropdown_body);
    let dropdown_wrapped = cell("dropdown_menu (click trigger)", dropdown_el, cx);

    // --- disclosure ---
    let entity_for_disc = cx.entity().clone();
    let disc_open = app.disclosure_open;
    let disc_label_str = if disc_open { "▼ " } else { "▶ " };
    let disc_trigger = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .child(label("ov-disc-arrow", disc_label_str, cx).render(cx))
        .child(label("ov-disc-text", cx.t("disclosure.title"), cx).strong(true).render(cx));
    let disc = disclosure("ov-disc", cx.t("disclosure.title"), cx)
        .open(disc_open)
        .on_toggle(move |_, _, cx| {
            entity_for_disc.update(cx, |s, _cx| {
                s.disclosure_open = !s.disclosure_open;
            });
        })
        .render(cx)
        .child(disc_trigger);
    let disc_with_body = if disc_open {
        disc.child(div().pl(px(16.)).child(label("ov-disc-body", cx.t("disclosure.body"), cx).render(cx)))
    } else {
        disc
    };
    let disc_wrapped = cell("disclosure (click to expand)", disc_with_body, cx);

    // --- overlay (scrim primitive; mirrors modal state) ---
    let overlay_el = overlay("ov-overlay", cx)
        .open(is_modal_open)
        .render(cx)
        .child(label("ov-overlay-info", "scrim follows modal state", cx).muted(true).render(cx));
    let overlay_wrapped = cell("overlay (scrim)", overlay_el, cx);

    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(modal_wrapped)
        .child(popover_wrapped)
        .child(tooltip_wrapped)
        .child(dropdown_wrapped)
        .child(disc_wrapped)
        .child(overlay_wrapped)
        .child(label("ov-summary", format!("dropdown: {} | menu: {}", app.dropdown_demo_value, app.menu_demo_value), cx).muted(true).render(cx))
}
