//! Popover Placement Demo App
//!
//! Displays two popovers side by side:
//! - Left trigger uses `BottomStart`
//! - Right trigger uses `BottomEnd`
//!
//! Also provides controls to toggle text direction.

use gpui::prelude::FluentBuilder;
use gpui::{
    Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

use yororen_ui::component::{PopoverPlacement, button, label, popover};
use yororen_ui::i18n::{I18nContext, Locale};
use yororen_ui::theme::ActiveTheme;

pub struct PopoverPlacementApp;

impl PopoverPlacementApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for PopoverPlacementApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Use keyed state to manage popover open/close (must borrow cx mutably first)
        let start_open: Entity<bool> =
            window.use_keyed_state("demo:popover:start:open", cx, |_, _| false);
        let end_open: Entity<bool> =
            window.use_keyed_state("demo:popover:end:open", cx, |_, _| false);

        let theme = cx.theme();
        let is_rtl = cx.i18n().text_direction().is_rtl();
        let dir_label = if is_rtl { "RTL (ar)" } else { "LTR (en)" };

        let title = div()
            .text_xl()
            .font_weight(FontWeight::BOLD)
            .text_color(theme.content.primary)
            .child("Popover Placement Demo");

        let description = div().text_sm().text_color(theme.content.secondary).child(
            "Click the buttons to open/close popovers. Verify BottomStart vs BottomEnd alignment.",
        );

        // Capture entity id so on_click can notify this component to re-render
        let entity_id = cx.entity_id();

        // Direction toggle button
        let dir_btn = button("toggle-dir")
            .child(format!("Switch to {}", if is_rtl { "LTR" } else { "RTL" }))
            .on_click(move |_ev, _window, cx| {
                let is_rtl = cx.i18n().text_direction().is_rtl();
                let mut next = cx.i18n().clone();
                next.set_locale(if is_rtl {
                    Locale::new("en").unwrap()
                } else {
                    Locale::new("ar").unwrap()
                });
                cx.set_global(next);
                cx.notify(entity_id);
            });

        // Info line showing current direction
        let info = div()
            .text_sm()
            .text_color(theme.content.secondary)
            .child(format!("Current direction: {}", dir_label));

        // BottomStart popover trigger (left side)
        let start_popover = {
            let is_open = *start_open.read(cx);
            popover("demo:popover:start")
                .placement(PopoverPlacement::BottomStart)
                .open(is_open)
                .trigger(
                    button("trigger:start")
                        .child("BottomStart")
                        .on_click({
                            let start_open = start_open.clone();
                            move |_ev, window, cx| {
                                start_open.update(cx, |open, _| *open = !*open);
                                window.refresh();
                            }
                        })
                        .into_any_element(),
                )
                .content(
                    div()
                        .p(px(16.))
                        .w(px(200.))
                        .child(label("This is BottomStart").strong(true))
                        .child(
                            label("Menu start edge aligns with trigger start edge.")
                                .wrap()
                                .muted(true),
                        )
                        .into_any_element(),
                )
                .on_close({
                    let start_open = start_open.clone();
                    move |_window, cx| {
                        start_open.update(cx, |open, _| *open = false);
                    }
                })
        };

        // BottomEnd popover trigger (right side)
        let end_popover = {
            let is_open = *end_open.read(cx);
            popover("demo:popover:end")
                .placement(PopoverPlacement::BottomEnd)
                .open(is_open)
                .trigger(
                    button("trigger:end")
                        .child("BottomEnd")
                        .on_click({
                            let end_open = end_open.clone();
                            move |_ev, window, cx| {
                                end_open.update(cx, |open, _| *open = !*open);
                                window.refresh();
                            }
                        })
                        .into_any_element(),
                )
                .content(
                    div()
                        .p(px(16.))
                        .w(px(200.))
                        .child(label("This is BottomEnd").strong(true))
                        .child(
                            label("Menu end edge aligns with trigger end edge.")
                                .wrap()
                                .muted(true),
                        )
                        .into_any_element(),
                )
                .on_close({
                    let end_open = end_open.clone();
                    move |_window, cx| {
                        end_open.update(cx, |open, _| *open = false);
                    }
                })
        };

        // Row of triggers — reverse order in RTL so "start" stays on the right
        let trigger_row = div()
            .flex()
            .when(is_rtl, |this| this.flex_row_reverse())
            .gap(px(24.))
            .items_start()
            .child(start_popover)
            .child(end_popover);

        // Tips
        let tips = div()
            .mt(px(24.))
            .p(px(16.))
            .rounded_md()
            .bg(theme.surface.raised)
            .border_1()
            .border_color(theme.border.default)
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(label("How to verify").strong(true))
            .child(
                label("1. Click both buttons — popovers should open/close.")
                    .muted(true)
                    .wrap(),
            )
            .child(
                label(
                    "2. Observe which edge of the menu aligns with the trigger (start vs end).",
                )
                .muted(true)
                .wrap(),
            )
            .child(
                label(
                    "3. Switch to RTL — the alignment should flip (BottomStart becomes right-aligned, BottomEnd becomes left-aligned).",
                )
                .muted(true)
                .wrap(),
            )
            .child(
                label(
                    "4. Resize the window to be narrow — the menu should be clamped inside the window bounds.",
                )
                .muted(true)
                .wrap(),
            );

        div()
            .size_full()
            .bg(theme.surface.base)
            .flex()
            .flex_col()
            .p(px(32.))
            .gap(px(16.))
            .child(title)
            .child(description)
            .child(info)
            .child(dir_btn)
            .child(trigger_row)
            .child(tips)
    }
}
