//! Root component for the i18n showcase demo.

use gpui::{
    Context, IntoElement, ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px,
};

use yororen_ui::component::{SelectOption, button, label, select, text_input};
use yororen_ui::i18n::{I18nContext, PlaceholderContext, PlaceholderKey, Translate};
use yororen_ui::theme::{ActionVariantKind, ActiveTheme};

use crate::state::I18nShowcaseState;
use crate::switch_locale;

pub struct I18nShowcaseApp;

impl I18nShowcaseApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for I18nShowcaseApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Snapshot everything we need from `cx` *immutably* before
        // building any element tree (so the immutable borrow is
        // released and we can still pass `cx` to click handlers later).
        let theme = cx.theme().clone();
        let state = cx.global::<I18nShowcaseState>();
        let current_locale = state.current(cx);
        let direction = cx.i18n().text_direction();
        let is_rtl = direction.is_rtl();

        // Resolve i18n keys and placeholders into owned values.
        let save_label = cx.t("common.save");
        let cancel_label = cx.t("common.cancel");
        let delete_label = cx.t("common.delete");
        let search_placeholder = cx
            .placeholder(PlaceholderKey::ComboBoxSearch)
            .unwrap_or_else(|| "Title…".into());

        // Build the locale switch row.
        let locale_row = build_locale_row(&current_locale);

        // Build the form (uses captured text from above).
        let form = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(label(save_label.clone()).strong(true))
            .child(text_input("i18n:title").placeholder(search_placeholder))
            .child(
                select("i18n:lang")
                    .option(SelectOption::new().value("en").label("English"))
                    .option(SelectOption::new().value("zh").label("中文"))
                    .option(SelectOption::new().value("ar").label("العربية")),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(button("i18n:cancel").child(cancel_label))
                    .child(
                        button("i18n:save")
                            .variant(ActionVariantKind::Primary)
                            .child(save_label),
                    )
                    .child(
                        button("i18n:delete")
                            .variant(ActionVariantKind::Danger)
                            .child(delete_label),
                    ),
            );

        let direction_label = if is_rtl { "RTL" } else { "LTR" };
        let direction_strip = div()
            .flex()
            .items_center()
            .justify_between()
            .child(label(format!("Direction: {direction_label}")))
            .child(label(format!("Locale: {current_locale}")));

        div()
            .size_full()
            .bg(theme.surface.base)
            .flex()
            .flex_col()
            .gap(px(20.))
            .p(px(28.))
            .child(label("i18n Showcase").strong(true).text_size(px(24.)))
            .child(label(
                "Switches locale at runtime via 3 external locale crates. \
                 i18n keys resolve through cx.t(). PlaceholderResolver slot \
                 is wired to a custom resolver. Switching to Arabic flips RTL.",
            ))
            .child(locale_row)
            .child(form)
            .child(direction_strip)
    }
}

/// Build the 3 locale-switch buttons. Click handlers call
/// `switch_locale` and refresh all open windows.
fn build_locale_row(current: &str) -> impl IntoElement {
    div()
        .flex()
        .gap(px(8.))
        .items_center()
        .child(locale_button("en", "English", current))
        .child(locale_button("zh", "中文", current))
        .child(locale_button("ar", "العربية", current))
}

fn locale_button(tag: &'static str, label_text: &'static str, current: &str) -> gpui::AnyElement {
    let variant = if tag == current {
        ActionVariantKind::Primary
    } else {
        ActionVariantKind::Neutral
    };
    button(format!("i18n:locale:{tag}"))
        .variant(variant)
        .child(label_text.to_string())
        .on_click(move |_ev, _window, cx| {
            // Clone the Entity handle out of the global so we can
            // release the immutable borrow before calling `set` (which
            // needs `&mut cx`).
            let locale_entity = cx.global::<I18nShowcaseState>().locale.clone();
            locale_entity.update(cx, |s, _cx| {
                *s = tag.to_string();
            });
            switch_locale(cx, tag);
            cx.refresh_windows();
        })
        .into_any_element()
}
