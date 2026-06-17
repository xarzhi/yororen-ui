//! Section 2 — Display. Each component is wrapped in a `cell`
//! that labels the component above it.

use gpui::{Context, Div, ParentElement, Styled, div, px};
use yororen_ui::i18n::Translate;
use yororen_ui::theme::ActiveTheme;

use yororen_ui::headless::badge::{BadgeVariant, badge};
use yororen_ui::headless::divider::divider;
use yororen_ui::headless::heading::HeadingLevel;
use yororen_ui::headless::heading::heading;
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
        .child(cell(
            cx.t("demo.display.cell_label_default"),
            label("lbl-default", cx.t("demo.display.label_default"), cx).render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_label_strong"),
            label("lbl-strong", cx.t("demo.display.label_strong"), cx)
                .strong(true)
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_label_muted"),
            label("lbl-muted", cx.t("demo.display.label_muted"), cx)
                .muted(true)
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_label_mono"),
            label("lbl-mono", cx.t("demo.display.label_mono"), cx)
                .mono(true)
                .render(cx),
            cx,
        ));

    // --- 4 heading levels ---
    let headings = div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(cell(
            cx.t("demo.display.cell_heading_h1"),
            heading(
                "hdg-1",
                HeadingLevel::H1,
                cx.t("demo.display.heading_h1"),
                cx,
            )
            .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_heading_h2"),
            heading(
                "hdg-2",
                HeadingLevel::H2,
                cx.t("demo.display.heading_h2"),
                cx,
            )
            .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_heading_h3"),
            heading(
                "hdg-3",
                HeadingLevel::H3,
                cx.t("demo.display.heading_h3"),
                cx,
            )
            .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_heading_h4"),
            heading(
                "hdg-4",
                HeadingLevel::H4,
                cx.t("demo.display.heading_h4"),
                cx,
            )
            .render(cx),
            cx,
        ));

    // --- 2 dividers ---
    let dividers = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            label("dvr-h-info", cx.t("demo.display.divider_h"), cx)
                .muted(true)
                .render(cx),
        )
        .child(divider("dvr-h1", cx).render(cx))
        .child(
            label("dvr-v-info", cx.t("demo.display.divider_v"), cx)
                .muted(true)
                .render(cx),
        )
        .child(divider("dvr-v1", cx).vertical().render(cx).h(px(24.)));

    // --- 5 badge variants ---
    let badges = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(8.))
        .child(cell(
            cx.t("button.neutral"),
            badge("bdg-n", cx.t("demo.display.badge_neutral"), cx)
                .variant(BadgeVariant::Neutral)
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("button.primary"),
            badge("bdg-s", cx.t("demo.display.badge_success"), cx)
                .variant(BadgeVariant::Success)
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("button.danger"),
            badge("bdg-w", cx.t("demo.display.badge_warning"), cx)
                .variant(BadgeVariant::Warning)
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("button.danger"),
            badge("bdg-d", cx.t("demo.display.badge_danger"), cx)
                .variant(BadgeVariant::Danger)
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("button.primary"),
            badge("bdg-i", cx.t("demo.display.badge_info"), cx)
                .variant(BadgeVariant::Info)
                .render(cx),
            cx,
        ));

    // --- tag: selected + closable ---
    let entity_for_tag_click = cx.entity().clone();
    let entity_for_tag_close = cx.entity().clone();
    let tag_closable_count = app.tag_closable_count;
    let tag_close_events_template = cx.t("demo.display.tag_close_events");
    let tag_row = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(8.))
        .child(cell(
            cx.t("demo.display.cell_tag_selected"),
            tag("tag-1", cx.t("demo.display.tag_toggle"), cx)
                .selected(app.tag_selected)
                .on_click(move |_, _, cx| {
                    entity_for_tag_click.update(cx, |s, _cx| {
                        s.tag_selected = !s.tag_selected;
                    });
                })
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_tag_closable"),
            tag("tag-2", cx.t("demo.display.tag_closable"), cx)
                .closable(true)
                .on_close(move |_, _, cx| {
                    entity_for_tag_close.update(cx, |s, _cx| {
                        s.tag_closable_count += 1;
                    });
                })
                .render(cx),
            cx,
        ))
        .child(
            label(
                "tag-closable-count",
                format!("{tag_close_events_template} {tag_closable_count}"),
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
        .child(cell(
            cx.t("demo.display.cell_skeleton_line"),
            skeleton("skl-line", cx).w(px(180.)).h(px(12.)).render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_skeleton_block"),
            skeleton("skl-block", cx)
                .block(true)
                .w(px(180.))
                .h(px(60.))
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_skeleton_block_sharp"),
            skeleton("skl-block-sharp", cx)
                .block(true)
                .block_sharp(true)
                .w(px(180.))
                .h(px(40.))
                .render(cx),
            cx,
        ));

    // --- progress ---
    let progress_label = cx.t("demo.display.loading").to_string();
    let progress_row = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(cell(
            cx.t("demo.display.cell_progress_determinate"),
            progress("prg-1", cx)
                .value(app.progress_value)
                .max(1.0)
                .label(progress_label)
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_progress_indeterminate"),
            progress("prg-indet", cx).indeterminate(true).render(cx),
            cx,
        ));

    // --- text + icon ---
    let icon_color = cx.theme().get_color("content.primary").unwrap_or_default();
    let text_row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .child(cell(
            cx.t("demo.display.cell_text"),
            text("txt-1", cx.t("demo.display.plain_text"), cx)
                .size(px(14.))
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_icon_check"),
            icon(
                "ico-1",
                yororen_ui::headless::icon::IconSource::Builtin("check".into()),
                cx,
            )
            .size(px(18.))
            .color(icon_color)
            .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.display.cell_icon_search"),
            icon(
                "ico-2",
                yororen_ui::headless::icon::IconSource::Builtin("search".into()),
                cx,
            )
            .size(px(18.))
            .color(icon_color)
            .render(cx),
            cx,
        ));

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
