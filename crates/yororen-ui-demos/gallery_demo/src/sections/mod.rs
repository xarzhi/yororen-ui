//! Per-category section functions. Each returns a section title
//! + the category's components as a single `impl IntoElement`.

mod actions;
mod controls;
mod display;
mod inputs;
mod lists;
mod notifications;
mod overlays;
mod surfaces;

pub use inputs::input_cell;

use gpui::{Context, Div, IntoElement, ParentElement, Styled, Window, div, px};

use yororen_ui::headless::heading::heading;
use yororen_ui::headless::heading::HeadingLevel;

use crate::state::GalleryApp;

/// Wrap a component in a labelled cell. The cell draws a small
/// muted `name` label above the component itself, in a
/// 1-pixel-bordered box, so the user can identify every
/// component in the gallery.
///
/// Use `name` like `"button / Primary"` or `"tag (closable)"`
/// — both the variant and the underlying headless primitive.
pub fn cell(
    name: &'static str,
    el: impl IntoElement,
    cx: &mut Context<GalleryApp>,
) -> Div {
    div()
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
    text: &'static str,
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
        .child(section_title("actions-title", "1. Actions", cx))
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
        .child(section_title("display-title", "2. Display", cx))
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
        .child(section_title("surfaces-title", "3. Surfaces", cx))
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
        .child(section_title("inputs-title", "4. Inputs", cx))
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
        .child(section_title("controls-title", "5. Controls", cx))
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
        .child(section_title("overlays-title", "6. Overlays", cx))
        .child(overlays::render(app, cx))
}

pub fn notifications(
    _app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("notifications-title", "7. Notifications", cx))
        .child(notifications::render(cx))
}

pub fn lists(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(section_title("lists-title", "8. Lists, Tables, Trees, Forms", cx))
        .child(lists::render(app, cx))
}
