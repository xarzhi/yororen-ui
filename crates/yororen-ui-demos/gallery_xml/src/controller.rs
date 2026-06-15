//! Controller — every business-logic method the XML
//! references. The XML itself stays purely declarative:
//! every `on_click={controller.foo}` resolves to a
//! method here via the macro's auto-wrap.
//!
//! `Controller` is `Clone` because the macro pre-clones
//! the receiver into a hygienic local per event handler
//! (see `yororen-ui-xml::codegen::auto_wrap_event_expr`).

use gpui::{App, ClickEvent, Entity, Window};

use yororen_ui::notification::center::{Notification, NotificationCenter, ToastKind};

use crate::state::{Counter, GalleryState, Section};

#[derive(Clone)]
pub struct Controller {
    state: Entity<GalleryState>,
}

impl Controller {
    pub fn new(state: Entity<GalleryState>) -> Self {
        Self { state }
    }

    /// The underlying state entity. The view uses it to
    /// observe changes (every mutation triggers a re-render).
    pub fn state(&self) -> Entity<GalleryState> {
        self.state.clone()
    }

    // -------- Toolbar actions --------

    /// Increment the toast counter and push a toast
    /// notification into the global `NotificationCenter`.
    /// The toast auto-dismisses after a few seconds
    /// (handled by the notification host).
    pub fn show_toast(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        let id = self.state.update(cx, |s, cx| {
            s.toast_count.update(cx, |c, cx| {
                c.value += 1;
                cx.notify();
            });
            cx.notify();
            s.toast_count.read(cx).value
        });
        let center = cx.global::<NotificationCenter>().clone();
        center.notify(
            Notification::new(format!("Toast #{}", id))
                .title("Gallery XML")
                .kind(ToastKind::Info),
            cx,
        );
    }

    /// Push a sticky notification — never auto-dismisses;
    /// the user must close it manually.
    pub fn show_notification(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        let id = self.state.update(cx, |s, cx| {
            s.toast_count.update(cx, |c, cx| {
                c.value += 1;
                cx.notify();
            });
            cx.notify();
            s.toast_count.read(cx).value
        });
        let center = cx.global::<NotificationCenter>().clone();
        center.notify(
            Notification::new(format!("Sticky #{}", id))
                .title("Sticky")
                .kind(ToastKind::Success)
                .sticky(true),
            cx,
        );
    }

    // -------- Actions section --------

    /// Toggle the demo toggle_button; record which
    /// "press" we're on so the footer label reflects the
    /// most recent action.
    pub fn press_toggle(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            let new_state = !*s.toggle_btn_selected.read(cx);
            s.toggle_btn_selected.update(cx, |b, cx| {
                *b = new_state;
                cx.notify();
            });
            s.last_action_label.update(cx, |l, cx| {
                *l = format!("toggle: {}", if new_state { "on" } else { "off" });
                cx.notify();
            });
            cx.notify();
        });
    }

    /// Bump progress by 10% on every press; clamps to 1.0.
    pub fn bump_progress(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.progress_value.update(cx, |p, cx| {
                *p = (*p + 0.1).min(1.0);
                cx.notify();
            });
            cx.notify();
        });
    }

    /// Reset progress to 0.0.
    pub fn reset_progress(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.progress_value.update(cx, |p, cx| {
                *p = 0.0;
                cx.notify();
            });
            cx.notify();
        });
    }

    /// Increment the closable-tag counter (shown in the
    /// tag's "x N" suffix).
    pub fn close_tag(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.tag_closable_count.update(cx, |c, cx| {
                c.value += 1;
                cx.notify();
            });
            cx.notify();
        });
    }

    // -------- Routing --------

    /// Switch the focused section. The XML's `<Match>`
    /// over `section` drives the highlight in the toolbar.
    ///
    /// The state `Entity` is cloned up-front so the
    /// returned closure doesn't borrow `&self` — that
    /// way the closure is `'static` and can be passed to
    /// event handlers stored in `Arc<dyn Fn>` slots.
    pub fn goto(&self, target: Section) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, cx| {
                s.section.update(cx, |cur, cx| {
                    *cur = target;
                    cx.notify();
                });
                cx.notify();
            });
        }
    }

    // -------- Generic helpers exposed to the XML --------

    /// Read-only accessor for the toast counter; the XML
    /// uses it for the footer label "Toasts: N".
    pub fn toast_count(&self) -> usize {
        // NOTE: the macro can't pass `cx` through here, so
        // this method is invoked synchronously inside the
        // render closure that does have `cx`. The view
        // surfaces the value as a captured local.
        0
    }

    /// Stub the controller uses to demonstrate the
    /// `unused` warning doesn't fire — every method
    /// the XML references must exist on `Controller`.
    pub fn _unused(_state: &GalleryState, _c: &Counter) {}
}
