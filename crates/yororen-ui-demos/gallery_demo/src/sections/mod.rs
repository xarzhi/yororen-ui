//! Per-category section functions. Each returns a section title
//! + the category's components as a single `impl IntoElement`.

mod actions;
mod controls;
mod display;
mod inputs;
mod lists;
mod overlays;
mod surfaces;

pub use inputs::input_cell;

use gpui::{Context, Div, IntoElement, ParentElement, Styled, Window, div, px};

use yororen_ui::headless::heading::heading;
use yororen_ui::headless::heading::HeadingLevel;
use yororen_ui::i18n::Translate;

use crate::state::GalleryApp;

/// Wrap a component in a labelled cell. The cell draws a small
/// muted `name` label above the component itself, in a
/// 1-pixel-bordered box, so the user can identify every
/// component in the gallery.
///
/// Use `name` like `"button / Primary"` or `"tag (closable)"`
/// — both the variant and the underlying headless primitive.
/// `name` is `impl Into<String>` so callers can pass either a
/// static literal (`"..."`) or the result of `cx.t("demo.foo")`
/// (which is `SharedString` and converts via the `From<SharedString>
/// for String` impl).
pub fn cell(
    name: impl Into<String>,
    el: impl IntoElement,
    cx: &mut Context<GalleryApp>,
) -> Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(2.))
        .p(px(8.))
        .rounded(px(6.))
        .border_1()
        .border_color(gpui::hsla(0.0, 0.0, 0.5, 0.15))
        .child(
            yororen_ui::headless::label::label("cmp-name", name, cx)
                .muted(true)
                .render(cx)
                .text_size(px(11.)),
        )
        .child(el)
}

fn section_title(
    id: &'static str,
    text: impl Into<String>,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    heading(id, HeadingLevel::H2, text, cx).apply(div()).mt(px(8.))
}

pub fn actions(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("actions-title", cx.t("demo.section_actions"), cx))
        .child(actions::render(app, cx))
}

pub fn display(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("display-title", cx.t("demo.section_display"), cx))
        .child(display::render(app, cx))
}

pub fn surfaces(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("surfaces-title", cx.t("demo.section_surfaces"), cx))
        .child(surfaces::render(app, cx))
}

pub fn inputs(
    app: &mut GalleryApp,
    window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("inputs-title", cx.t("demo.section_inputs"), cx))
        .child(inputs::render(app, window, cx))
}

pub fn controls(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("controls-title", cx.t("demo.section_controls"), cx))
        .child(controls::render(app, cx))
}

pub fn overlays(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("overlays-title", cx.t("demo.section_overlays"), cx))
        .child(overlays::render(app, cx))
}

pub fn lists(
    app: &mut GalleryApp,
    window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("lists-title", cx.t("demo.section_lists"), cx))
        .child(lists::render(app, window, cx))
}
