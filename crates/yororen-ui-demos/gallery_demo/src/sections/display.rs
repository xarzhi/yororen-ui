//! Section 2 — Display. Each component is wrapped in a `cell`
//! that labels the component above it.

use gpui::{Context, Div, ParentElement, Styled, div, hsla, px};

use yororen_ui::headless::badge::{BadgeVariant, badge};
use yororen_ui::headless::divider::divider;
use yororen_ui::headless::heading::heading;
use yororen_ui::headless::heading::HeadingLevel;
use yororen_ui::headless::icon::icon;
use yororen_ui::headless::label::label;
use yororen_ui::headless::progress::progress;
use yororen_ui::headless::skeleton::skeleton;
use yororen_ui::headless::tag::tag;
use yororen_ui::headless::text::text;

use crate::sections::cell;
use crate::state::GalleryApp;

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Div {
    // --- 4 label styles ---
    let labels = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(12.))
        .child(cell("label / default", label("lbl-default", "Default", cx).render(cx), cx))
        .child(cell("label / strong", label("lbl-strong", "Strong", cx).strong(true).render(cx), cx))
        .child(cell("label / muted", label("lbl-muted", "Muted", cx).muted(true).render(cx), cx))
        .child(cell("label / mono", label("lbl-mono", "Mono 0x1F", cx).mono(true).render(cx), cx));

    // --- 4 heading levels ---
    let headings = div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(cell("heading / H1", heading("h-1", HeadingLevel::H1, "H1 heading", cx).render(cx), cx))
        .child(cell("heading / H2", heading("h-2", HeadingLevel::H2, "H2 heading", cx).render(cx), cx))
        .child(cell("heading / H3", heading("h-3", HeadingLevel::H3, "H3 heading", cx).render(cx), cx))
        .child(cell("heading / H4", heading("h-4", HeadingLevel::H4, "H4 heading", cx).render(cx), cx));

    // --- 2 dividers ---
    let dividers = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(label("d-h-info", "divider / horizontal", cx).muted(true).render(cx))
        .child(divider("d-h1", cx).apply(div().w_full().h(px(1.)).bg(hsla(0.0, 0.0, 0.5, 0.4))))
        .child(label("d-v-info", "divider / vertical", cx).muted(true).render(cx))
        .child(divider("d-v1", cx).vertical().apply(div().h(px(24.)).w(px(1.)).bg(hsla(0.0, 0.0, 0.5, 0.4))));

    // --- 5 badge variants ---
    let badges = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(8.))
        .child(cell("badge / Neutral", badge("bd-n", "Neutral", cx).variant(BadgeVariant::Neutral).apply(div().px(px(8.)).py(px(2.)).rounded(px(4.))), cx))
        .child(cell("badge / Success", badge("bd-s", "Success", cx).variant(BadgeVariant::Success).apply(div().px(px(8.)).py(px(2.)).rounded(px(4.))), cx))
        .child(cell("badge / Warning", badge("bd-w", "Warning", cx).variant(BadgeVariant::Warning).apply(div().px(px(8.)).py(px(2.)).rounded(px(4.))), cx))
        .child(cell("badge / Danger", badge("bd-d", "Danger", cx).variant(BadgeVariant::Danger).apply(div().px(px(8.)).py(px(2.)).rounded(px(4.))), cx))
        .child(cell("badge / Info", badge("bd-i", "Info", cx).variant(BadgeVariant::Info).apply(div().px(px(8.)).py(px(2.)).rounded(px(4.))), cx));

    // --- tag: selected + closable ---
    let entity_for_tag_click = cx.entity().clone();
    let entity_for_tag_close = cx.entity().clone();
    let tag_closable_count = app.tag_closable_count;
    let tag_row = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(8.))
        .child(cell(
            "tag (selected)",
            tag("tg-1", "Tap to toggle", cx)
                .selected(app.tag_selected)
                .on_click(move |_, _, cx| {
                    entity_for_tag_click.update(cx, |s, _cx| {
                        s.tag_selected = !s.tag_selected;
                    });
                })
                .apply(div().px(px(8.)).py(px(2.)).rounded(px(4.))),
            cx,
        ))
        .child(cell(
            "tag / closable",
            tag("tg-2", "Closable", cx)
                .closable(true)
                .on_close(move |_, _, cx| {
                    entity_for_tag_close.update(cx, |s, _cx| {
                        s.tag_closable_count += 1;
                    });
                })
                .apply(div().px(px(8.)).py(px(2.)).rounded(px(4.))),
            cx,
        ))
        .child(
            label(
                "tg-closable-count",
                format!("tag close events: {tag_closable_count}"),
                cx,
            )
            .muted(true)
            .render(cx),
        );

    // --- skeleton: line + block ---
    let skeletons = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(cell("skeleton / line", skeleton("sk-line", cx).apply(div().w(px(180.)).h(px(12.)).rounded(px(4.))), cx))
        .child(cell("skeleton / block", skeleton("sk-block", cx).block(true).apply(div().w(px(180.)).h(px(60.)).rounded(px(4.))), cx))
        .child(cell("skeleton / block sharp", skeleton("sk-block-sharp", cx).block(true).block_sharp(true).apply(div().w(px(180.)).h(px(40.))), cx));

    // --- progress ---
    let progress_row = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(cell("progress (determinate)", progress("prg-1", cx).value(app.progress_value).max(1.0).label("Loading…").apply(div().w(px(220.)).h(px(8.)).rounded(px(4.))), cx))
        .child(cell("progress (indeterminate)", progress("prg-indet", cx).indeterminate(true).apply(div().w(px(220.)).h(px(8.)).rounded(px(4.))), cx));

    // --- text + icon ---
    let text_row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .child(cell("text", text("tx-1", "Plain text via `text`", cx).size(px(14.)).apply(div()), cx))
        .child(cell("icon (check)", icon("ic-1", yororen_ui::headless::icon::IconSource::Builtin("check".into()), cx).size(px(18.)).color(gpui::rgb(0x0A0A0A)).render(), cx))
        .child(cell("icon (search)", icon("ic-2", yororen_ui::headless::icon::IconSource::Builtin("search".into()), cx).size(px(18.)).color(gpui::rgb(0x0A0A0A)).render(), cx));

    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(labels)
        .child(headings)
        .child(dividers)
        .child(badges)
        .child(tag_row)
        .child(skeletons)
        .child(progress_row)
        .child(text_row)
}
