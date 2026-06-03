//! Root component for the modal a11y demo.

use std::sync::Arc;

use gpui::{Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, px};

use yororen_ui::component::{
    OverlayCloseCallback, OverlayCloseReason, button, label, modal, modal_actions_row, overlay,
};
use yororen_ui::theme::{ActionVariantKind, ActiveTheme};

use crate::state::ModalA11yState;

pub struct ModalA11yApp;

impl ModalA11yApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ModalA11yApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<ModalA11yState>();
        let visibility = state.visibility(cx);
        let close_log = state.close_log.read(cx);
        let theme = cx.theme();
        let close_log_label: SharedString = if close_log.entries.is_empty() {
            "Recent closes: (none yet)".to_string().into()
        } else {
            let parts: Vec<String> = close_log
                .entries
                .iter()
                .map(|(r, _)| match r {
                    OverlayCloseReason::ScrimClick => "scrim",
                    OverlayCloseReason::Escape => "escape",
                    OverlayCloseReason::Programmatic => "button",
                })
                .map(|s| s.to_string())
                .collect();
            format!("Recent closes: {}", parts.join(", ")).into()
        };

        // === Button row to open each modal kind. ===
        let open_standard_entity = state.visibility.clone();
        let open_required_entity = state.visibility.clone();
        let open_no_lock_entity = state.visibility.clone();

        let mut root = div()
            .size_full()
            .bg(theme.surface.canvas)
            .flex()
            .flex_col()
            .gap(px(20.))
            .p(px(24.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(label("Modal a11y Showcase").strong(true).text_size(px(24.)))
                    .child(label(
                        "Three modal kinds. Standard closes on Esc / scrim / button. Required \
                         (non-dismissable) only closes on a button. No-scroll-lock variant is \
                         the same as standard but does not lock body scroll. Watch the 'recent \
                         closes' label update as you dismiss them.",
                    )),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(
                        button("open-standard")
                            .variant(ActionVariantKind::Primary)
                            .child("Open standard modal")
                            .on_click(move |_ev, _w, cx| {
                                open_standard_entity.update(cx, |v, _cx| {
                                    v.standard = true;
                                });
                                cx.refresh_windows();
                            }),
                    )
                    .child(
                        button("open-required")
                            .variant(ActionVariantKind::Danger)
                            .child("Open required modal (non-dismissable)")
                            .on_click(move |_ev, _w, cx| {
                                open_required_entity.update(cx, |v, _cx| {
                                    v.required = true;
                                });
                                cx.refresh_windows();
                            }),
                    )
                    .child(
                        button("open-no-scroll")
                            .variant(ActionVariantKind::Neutral)
                            .child("Open no-scroll-lock modal")
                            .on_click(move |_ev, _w, cx| {
                                open_no_lock_entity.update(cx, |v, _cx| {
                                    v.no_scroll_lock = true;
                                });
                                cx.refresh_windows();
                            }),
                    ),
            )
            .child(label(close_log_label).muted(true));

        // === Modals, all wrapped in Overlay for the v0.5 a11y stack. ===

        // 1. Standard modal.
        if visibility.standard {
            let visibility = state.visibility.clone();
            let log = state.close_log.clone();
            // Single close callback shared by the overlay
            // (scrim click / Esc) and the modal's Cancel/OK
            // buttons. Buttons invoke it with
            // `OverlayCloseReason::Programmatic`.
            let on_close: OverlayCloseCallback = Arc::new(move |reason, _w, cx| {
                log.update(cx, |log, _| log.push(*reason));
                visibility.update(cx, |v, _| v.standard = false);
                cx.refresh_windows();
            });
            let on_close_for_overlay = on_close.clone();
            root = root.child(
                overlay("a11y:overlay:standard")
                    .open(true)
                    .on_close(move |reason, w, cx| on_close_for_overlay(reason, w, cx))
                    .child(build_standard_modal(
                        "Standard modal",
                        "Click outside or press Esc to close. Cancel / OK also work.",
                        on_close,
                    )),
            );
        }

        // 2. Non-dismissable (required) modal.
        if visibility.required {
            let visibility = state.visibility.clone();
            let log = state.close_log.clone();
            let on_close: OverlayCloseCallback = Arc::new(move |reason, _w, cx| {
                log.update(cx, |log, _| log.push(*reason));
                visibility.update(cx, |v, _| v.required = false);
                cx.refresh_windows();
            });
            let on_close_for_overlay = on_close.clone();
            root = root.child(
                overlay("a11y:overlay:required")
                    .open(true)
                    .dismiss_on_escape(false)
                    .dismiss_on_scrim(false)
                    .on_close(move |reason, w, cx| on_close_for_overlay(reason, w, cx))
                    .child(build_standard_modal(
                        "Required modal",
                        "This modal does not close on Esc or scrim click. Use the buttons below.",
                        on_close,
                    )),
            );
        }

        // 3. No-scroll-lock modal.
        if visibility.no_scroll_lock {
            let visibility = state.visibility.clone();
            let log = state.close_log.clone();
            let on_close: OverlayCloseCallback = Arc::new(move |reason, _w, cx| {
                log.update(cx, |log, _| log.push(*reason));
                visibility.update(cx, |v, _| v.no_scroll_lock = false);
                cx.refresh_windows();
            });
            let on_close_for_overlay = on_close.clone();
            root = root.child(
                overlay("a11y:overlay:no-scroll")
                    .open(true)
                    .lock_scroll(false)
                    .on_close(move |reason, w, cx| on_close_for_overlay(reason, w, cx))
                    .child(build_standard_modal(
                        "No-scroll-lock modal",
                        "Same as standard but does not lock body scroll. The user can still scroll the page behind the modal.",
                        on_close,
                    )),
            );
        }

        root
    }
}

fn build_standard_modal(
    title: &str,
    body: &str,
    on_close: OverlayCloseCallback,
) -> gpui::AnyElement {
    let title = title.to_string();
    let body = body.to_string();
    let modal_id = format!("a11y:modal:{}", title);
    let cancel_id = format!("a11y:modal:{}:cancel", title);
    let ok_id = format!("a11y:modal:{}:ok", title);
    // Each button gets its own clone of the callback so we can
    // route through the same `on_close` (with
    // `OverlayCloseReason::Programmatic`) that the overlay uses
    // for scrim click and Esc. This way the close log records
    // "button" for button dismisses, and the visibility flag is
    // cleared through one place.
    let on_close_cancel = on_close.clone();
    let on_close_ok = on_close;
    modal()
        .id(modal_id)
        .title(title.clone())
        .aria_label(title)
        .content(label(body))
        .actions(modal_actions_row(
            yororen_ui::i18n::TextDirection::Ltr,
            [
                button(cancel_id)
                    .child("Cancel")
                    .on_click(move |_ev, w, cx| {
                        on_close_cancel(&OverlayCloseReason::Programmatic, w, cx);
                    })
                    .into_any_element(),
                button(ok_id)
                    .variant(ActionVariantKind::Primary)
                    .child("OK")
                    .on_click(move |_ev, w, cx| {
                        on_close_ok(&OverlayCloseReason::Programmatic, w, cx);
                    })
                    .into_any_element(),
            ],
        ))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ModalVisibility;
    use yororen_ui::component::modal_dialog;

    #[test]
    fn build_standard_modal_returns_any_element() {
        let on_close: OverlayCloseCallback =
            Arc::new(|_reason, _w, _cx| {});
        let _: gpui::AnyElement = build_standard_modal("t", "b", on_close);
    }

    #[test]
    fn modal_visibility_starts_all_false() {
        let v = ModalVisibility::default();
        assert!(!v.standard);
        assert!(!v.required);
        assert!(!v.no_scroll_lock);
    }

    /// G-γ: `modal_dialog(id)` is the one-line API. The
    /// resulting `ModalShell` is `IntoElement` so the caller can
    /// embed it in a parent layout directly. We exercise both
    /// paths: raw construction, and `.into_any_element()` for
    /// embedding in an existing element tree.
    #[test]
    fn modal_dialog_one_line_returns_any_element() {
        let shell = modal_dialog("one-line-modal")
            .open(true)
            .title("Hello")
            .content(label("Body"))
            .on_close(|_reason, _w, _cx| {});
        // The shell is IntoElement, so we can convert it for
        // embedding in a parent layout.
        let _any: gpui::AnyElement = shell.into_any_element();
    }
}
