//! Section 3 — Surfaces. Each component is wrapped in a
//! labelled `cell` so the user can identify every instance.

use gpui::{Context, Div, ParentElement, Styled, div, hsla, px};

use yororen_ui::headless::avatar::avatar;
use yororen_ui::headless::button::button;
use yororen_ui::headless::card::card;
use yororen_ui::headless::empty_state::empty_state;
use yororen_ui::headless::focus_ring::focus_ring;
use yororen_ui::headless::icon::IconSource;
use yororen_ui::headless::image::image;
use yororen_ui::headless::image::ImageSource;
use yororen_ui::headless::keybinding_display::keybinding_display;
use yororen_ui::headless::label::label;
use yororen_ui::headless::panel::panel;
use yororen_ui::headless::shortcut_hint::shortcut_hint;
use yororen_ui::i18n::Translate;

use crate::sections::cell;
use crate::state::GalleryApp;

pub fn render(_app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Div {
    // --- avatars ---
    let avatars = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .child(cell(cx.t("demo.surfaces.cell_avatar_initials_status"), avatar("av-1", cx).initials(cx.t("demo.surfaces.avatar_initials_ab")).size(px(48.)).circle(true).has_status(true).render(cx), cx))
        .child(cell(cx.t("demo.surfaces.cell_avatar_initials_square"), avatar("av-2", cx).initials(cx.t("demo.surfaces.avatar_initials_cd")).size(px(48.)).circle(false).render(cx), cx))
        .child(cell(cx.t("demo.surfaces.cell_avatar_name"), avatar("av-3", cx).name(cx.t("demo.surfaces.avatar_name")).size(px(48.)).render(cx), cx));

    // --- card (interactive) ---
    let card_el = card("card-1", cx)
        .interactive(true)
        .render(cx)
        .w(px(220.))
        .child(label("card-title", cx.t("demo.surfaces.card_title"), cx).strong(true).render(cx))
        .child(label("card-body", cx.t("demo.surfaces.card_body"), cx).muted(true).render(cx))
        .child(button("card-btn", cx).on_click(|_, _, _| {}).render(cx).child(cx.t("demo.surfaces.card_action")));
    let card_wrapped = cell(cx.t("demo.surfaces.cell_card"), card_el, cx);

    // --- panel with title ---
    let panel_el = panel("panel-1", cx)
        .title(cx.t("demo.surfaces.panel_title"))
        .padded(true)
        .render(cx)
        .w(px(280.))
        .child(label("panel-body", cx.t("demo.surfaces.panel_body"), cx).render(cx));
    let panel_wrapped = cell(cx.t("demo.surfaces.cell_panel"), panel_el, cx);

    // --- empty_state ---
    let empty = empty_state("es-1", cx)
        .icon(IconSource::Builtin("info".into()))
        .title(cx.t("demo.surfaces.empty_title"))
        .description(cx.t("demo.surfaces.empty_desc"))
        .render(cx)
        .w(px(280.))
        .child(label("es-extra", cx.t("demo.surfaces.empty_custom_child"), cx).muted(true).render(cx));
    let empty_wrapped = cell(cx.t("demo.surfaces.cell_empty"), empty, cx);

    // --- focus_ring wrapping a button ---
    let ring_target = button("focus-btn", cx)
        .on_click(|_, _, _| {})
        .render(cx)
        .child(cx.t("demo.surfaces.focusable"));
    let ring_focus_handle = cx.focus_handle();
    let ringed = focus_ring("ring-1", &ring_focus_handle, cx)
        .render(cx)
        .child(ring_target);
    let ring_wrapped = cell(cx.t("demo.surfaces.cell_focus_ring"), ringed, cx);

    // --- image (resource path; the file is not bundled in the
    //     demo, but the headless contract is shown via the
    //     placeholder background). ---
    let img = image("img-1", ImageSource::Resource("images/sample.png".into()), cx)
        .alt(cx.t("demo.surfaces.image_alt"))
        .render(cx)
        .w(px(120.))
        .h(px(80.))
        .bg(hsla(0.0, 0.0, 0.85, 1.0))
        .rounded(px(4.))
        .border_1();
    let img_wrapped = cell(cx.t("demo.surfaces.cell_image"), img, cx);

    // --- keybinding_display ---
    let kbd_disp = keybinding_display("kbd-1", vec!["Ctrl".to_string(), "S".to_string()], cx)
        .render(cx);
    let kbd_wrapped = cell(cx.t("demo.surfaces.cell_keybinding_display"), kbd_disp, cx);

    // --- shortcut_hint ---
    let sh = shortcut_hint("sh-1", cx.t("demo.surfaces.shortcut_save_caption"), vec!["Cmd".to_string(), "S".to_string()], cx)
        .render(cx);
    let sh_wrapped = cell(cx.t("demo.surfaces.cell_shortcut_hint"), sh, cx);

    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(avatars)
        .child(div().flex().flex_row().flex_wrap().gap(px(12.)).child(card_wrapped).child(panel_wrapped).child(empty_wrapped))
        .child(div().flex().flex_row().flex_wrap().gap(px(12.)).items_center().child(ring_wrapped).child(img_wrapped))
        .child(div().flex().flex_col().gap(px(8.)).child(kbd_wrapped).child(sh_wrapped))
}
