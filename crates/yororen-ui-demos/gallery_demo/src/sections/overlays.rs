//! Section 6 — Overlays. Each component is wrapped in a
//! labelled `cell`.

use gpui::{
    Context, Div, IntoElement, ParentElement, StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui::headless::button::button;
use yororen_ui::headless::disclosure::disclosure;
use yororen_ui::headless::dropdown_menu::dropdown_menu;
use yororen_ui::headless::label::label;
use yororen_ui::headless::menu::menu;
use yororen_ui::headless::overlay::overlay;
use yororen_ui::headless::popover::popover;
use yororen_ui::headless::tooltip::tooltip;
use yororen_ui::i18n::Translate;

use crate::sections::cell;
use crate::state::GalleryApp;

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Div {
    // --- modal trigger only ---
    //
    // The modal scrim/panel itself is rendered by
    // `gallery_app::render` at the `scroll_root` level so that
    // (1) when the overlays row is virtualized off-screen, the
    // open modal stays visible, and (2) `.absolute().inset_0()`
    // pins to the window-spanning `scroll_root` rather than the
    // (smaller) row container.
    let modal_state_for_btn = app.modal_state.clone();
    let open_modal_btn = button("ov-modal-open", cx)
        .on_click(move |_, _, cx| {
            modal_state_for_btn.update(cx, |st, _cx| st.open());
        })
        .render(cx)
        .child("Open modal");
    let modal_wrapped = cell(
        "modal (click to open)",
        div().flex().flex_col().child(open_modal_btn),
        cx,
    );
    let is_modal_open = app.modal_state.read(cx).open;

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
    // The popover's body is a real menu (with items). The
    // menu renderer iterates `menu_state.items` to draw the
    // rows, so we only need to set up the on_select handler
    // and pass the menu element to `popover().content(...)`.
    let entity_for_menu = cx.entity().clone();
    let menu_state = app.menu_state.clone();
    let popover_state_for_close = popover_state.clone();
    menu_state.update(cx, |s, _cx| {
        s.set_on_select(move |id, _w, cx| {
            let id_s = id.to_string();
            entity_for_menu.update(cx, |s, _cx| {
                s.menu_demo_value = id_s;
            });
            // Selecting a menu item also closes the popover
            // so the menu dismisses itself after the click.
            popover_state_for_close.update(cx, |s, _cx| s.close());
        });
    });
    let menu_el = menu("ov-menu", menu_state.clone()).render(cx);
    // Empty placeholder so the popover's content slot is
    // always populated; the renderer decides visibility by
    // `popover_state.is_open()`.
    let popover_content = if popover_is_open {
        menu_el.into_any_element()
    } else {
        div().into_any_element()
    };
    let popover_el = popover("ov-popover", popover_state.clone())
        .trigger(popover_trigger.into_any_element())
        .content(popover_content)
        .render(cx);
    let popover_wrapped = cell("popover (click trigger)", popover_el, cx);

    // --- tooltip ---
    // The trigger stays in flow; the tooltip text floats
    // over it via `gpui::deferred` when `tooltip_state.is_open()`.
    // The headless state machine drives open/close; the demo
    // wires `on_hover` on the trigger so hover/focus state
    // toggles `tooltip_state`.
    let tooltip_state = app.tooltip_state.clone();
    let tooltip_state_for_hover = tooltip_state.clone();
    let tooltip_trigger = label("ov-tt-target", "Hover or focus me", cx)
        .render(cx)
        .on_hover(move |hovered, _w, cx| {
            if *hovered {
                tooltip_state_for_hover.update(cx, |s, _cx| s.open());
            } else {
                tooltip_state_for_hover.update(cx, |s, _cx| s.close());
            }
        })
        .into_any_element();
    let tooltip_el = tooltip("ov-tooltip", cx.t("tooltip.text"), tooltip_state.clone())
        .trigger(tooltip_trigger)
        .render(cx);
    let tooltip_wrapped = cell("tooltip (hover target)", tooltip_el, cx);

    // --- dropdown_menu ---
    // Body is a real menu that iterates `dropdown_state.items`
    // (Cut / Copy / Paste / separator / Select all). The
    // dropdown renderer wraps the menu in a floating panel
    // when `dropdown_state.is_open()`. We reuse `menu_state`
    // for the body's rendering — items are seeded on
    // `dropdown_state`, but the menu's own on_select wiring
    // is what fires per-click (which in turn calls the
    // dropdown's on_select to update `dropdown_demo_value`).
    let entity_for_dropdown = cx.entity().clone();
    let dropdown_state = app.dropdown_state.clone();
    let entity_for_dropdown_for_menu = entity_for_dropdown.clone();
    let menu_state_for_dropdown = app.menu_state.clone();
    menu_state_for_dropdown.update(cx, |s, _cx| {
        s.set_on_select(move |id, _w, cx| {
            let id_s = id.to_string();
            entity_for_dropdown_for_menu.update(cx, |app, _cx| {
                app.dropdown_demo_value = id_s.clone();
                app.menu_demo_value = id_s;
            });
        });
    });
    let _ = entity_for_dropdown; // silence unused
    let _ = dropdown_state; // silence unused
    let dropdown_state_for_btn = app.dropdown_state.clone();
    let dropdown_trigger = button("ov-dropdown-trigger", cx)
        .on_click(move |_, _, cx| {
            dropdown_state_for_btn.update(cx, |st, _cx| st.toggle());
        })
        .render(cx)
        .child("Open menu ▾");
    let dropdown_menu_el = menu("ov-dropdown-body", menu_state_for_dropdown.clone()).render(cx);
    let dropdown_content = if app.dropdown_state.read(cx).open {
        dropdown_menu_el.into_any_element()
    } else {
        div().into_any_element()
    };
    let dropdown_el = dropdown_menu("ov-dropdown", app.dropdown_state.clone())
        .trigger(dropdown_trigger.into_any_element())
        .content(dropdown_content)
        .render(cx);
    let dropdown_wrapped = cell("dropdown_menu (click trigger)", dropdown_el, cx);

    // --- disclosure ---
    // The renderer (default or brutalism) draws the trigger
    // (chevron + title). The demo only appends the expanded
    // body as a child, eliminating the previous "double trigger"
    // artifact.
    let entity_for_disc = cx.entity().clone();
    let disc_open = app.disclosure_open;
    let disc = disclosure("ov-disc", cx.t("disclosure.title"), cx)
        .open(disc_open)
        .on_toggle(move |_, _, cx| {
            entity_for_disc.update(cx, |s, _cx| {
                s.disclosure_open = !s.disclosure_open;
            });
        })
        .render(cx);
    let disc_with_body = if disc_open {
        disc.child(
            div()
                .pl(px(16.))
                .child(label("ov-disc-body", cx.t("disclosure.body"), cx).render(cx)),
        )
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
