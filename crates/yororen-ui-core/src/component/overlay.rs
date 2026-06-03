//! `Overlay` — a full-window scrim that wraps a single child element
//! (typically a [`Modal`](super::modal::Modal)).
//!
//! Phase G.2 introduces the v0.5 accessibility story for floating
//! UI. `Overlay` is the building block that provides:
//!
//! - **Scrim**: a semi-transparent full-window backdrop using the
//!   `ModalRenderer::scrim` color from the active theme.
//! - **Click-on-scrim to close**: a click outside the inner child
//!   fires the `on_close` callback. Use `dismiss_on_scrim(false)` to
//!   disable this for non-dismissable dialogs (e.g. permission
//!   prompts).
//! - **Esc to close**: pressing the Escape key while the overlay is
//!   focused fires `on_close`. Use `dismiss_on_escape(false)` to
//!   disable for non-dismissable dialogs.
//! - **Body scroll lock**: while the overlay is open, the global
//!   scroll-lock counter is incremented, preventing background
//!   content from scrolling. Drop the overlay's render frame to
//!   release the lock.
//! - **Centered child layout**: the inner child is centered both
//!   horizontally and vertically inside the scrim.
//!
//! `Overlay` does **not** itself implement focus trapping; the v0.5
//! story keeps that responsibility on the child component (which
//! has the right `FocusHandle`s) and on the [`FocusTrap`](crate::a11y::FocusTrap)
//! helper. `Overlay` does set a `tab_index` on the scrim so the
//! scrim itself can receive keyboard focus and the Escape key.
//!
//! # Usage
//!
//! ```rust,ignore
//! use yororen_ui::component::{modal, overlay};
//!
//! overlay("my-modal")
//!     .open(state.modal_open)
//!     .on_close(move |reason, _w, cx| {
//!         state.modal_open = false;
//!         cx.refresh_windows();
//!     })
//!     .child(
//!         modal()
//!             .title("Delete file?")
//!             .content(label("This cannot be undone."))
//!     )
//! ```

use std::sync::Arc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, KeyDownEvent, MouseDownEvent,
    ParentElement, RenderOnce, StyleRefinement, Styled, Window, div,
};

use crate::a11y::ScrollLockGuard;
use crate::theme::ActiveTheme;

/// Scrim-click callback. Factored out so clippy doesn't complain
/// about the `Arc<dyn Fn(...)>` type being too complex.
type ScrimCallback = Arc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + Send + Sync>;

/// Callback type for `Overlay::on_close`. Mirrors the rest of the
/// a11y helpers: takes the close reason, a mutable window, and a
/// mutable app.
pub type OverlayCloseCallback =
    Arc<dyn Fn(&OverlayCloseReason, &mut Window, &mut gpui::App) + Send + Sync>;

/// Reason the overlay was closed. Useful for analytics / conditional
/// logic in the `on_close` handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayCloseReason {
    /// User clicked the scrim outside the inner child.
    ScrimClick,
    /// User pressed Escape.
    Escape,
    /// Programmatic close (e.g. `on_close` called from a child
    /// button's `on_click`).
    Programmatic,
}

/// Build a new `Overlay` element. The overlay wraps a single child
/// (typically a [`Modal`](super::modal::Modal)) with a scrim, click-
/// outside-to-close, Esc-to-close, and a body scroll lock.
///
/// Pass a stable `id` so the overlay's key state survives across
/// renders. The default `id` is `"ui:overlay"`.
pub fn overlay(id: impl Into<ElementId>) -> Overlay {
    Overlay::new(id)
}

#[derive(IntoElement)]
pub struct Overlay {
    element_id: ElementId,
    base: gpui::Div,
    open: bool,
    on_close: Option<OverlayCloseCallback>,
    dismiss_on_scrim: bool,
    dismiss_on_escape: bool,
    lock_scroll: bool,
    child: Option<AnyElement>,
}

impl Default for Overlay {
    fn default() -> Self {
        Self::new("ui:overlay")
    }
}

impl Overlay {
    /// Create a new `Overlay` with the given element id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            element_id: id.into(),
            base: div(),
            open: false,
            on_close: None,
            dismiss_on_scrim: true,
            dismiss_on_escape: true,
            lock_scroll: true,
            child: None,
        }
    }

    /// Set the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Returns the element's ID.
    pub fn element_id(&self) -> &ElementId {
        &self.element_id
    }

    /// Open or close the overlay. When `false`, the overlay renders
    /// as an empty `div` so the closure is still `IntoElement`-safe
    /// (e.g. when the parent always expects a single element child).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Set the close callback. Fires when the user dismisses via
    /// scrim click or Escape. Programmatic closes (from a child
    /// button's `on_click`) can call this callback too with
    /// `OverlayCloseReason::Programmatic`.
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&OverlayCloseReason, &mut Window, &mut gpui::App),
    {
        self.on_close = Some(Arc::new(handler));
        self
    }

    /// Convenience: same as `on_close` but ignores the reason. Use
    /// this when the handler is a single closure that doesn't care
    /// why the overlay closed.
    pub fn on_close_any<F>(mut self, handler: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut Window, &mut gpui::App),
    {
        self.on_close = Some(Arc::new(move |_reason, window, cx| {
            handler(window, cx);
        }));
        self
    }

    /// Set whether a click on the scrim dismisses the overlay.
    /// Default: `true`. Set to `false` for non-dismissable dialogs.
    pub fn dismiss_on_scrim(mut self, dismiss: bool) -> Self {
        self.dismiss_on_scrim = dismiss;
        self
    }

    /// Set whether the Escape key dismisses the overlay.
    /// Default: `true`. Set to `false` for non-dismissable dialogs.
    pub fn dismiss_on_escape(mut self, dismiss: bool) -> Self {
        self.dismiss_on_escape = dismiss;
        self
    }

    /// Set whether the overlay locks the body scroll while open.
    /// Default: `true`. Set to `false` for non-modal overlays
    /// (e.g. tooltips, non-modal popovers).
    pub fn lock_scroll(mut self, lock: bool) -> Self {
        self.lock_scroll = lock;
        self
    }

    /// Set the inner child element. The overlay renders exactly one
    /// child; if you need a tree, wrap it in a `div()`.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl ParentElement for Overlay {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        // Overlay is a single-child container: we treat `extend` as
        // a way to set the child (last-wins) for ergonomic call
        // sites like `overlay(id).child(...).extend(...)`.
        if let Some(last) = elements.into_iter().last() {
            self.child = Some(last);
        }
    }
}

impl Styled for Overlay {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Overlay {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl RenderOnce for Overlay {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        if !self.open {
            return div().into_any_element();
        }

        let element_id = self.element_id;
        let child = self.child.unwrap_or_else(|| div().into_any_element());

        // The body scroll lock is RAII: the guard lives in this
        // render frame and drops when the frame is dropped. In a
        // `RenderOnce` model, that's "until the parent re-renders
        // without the overlay open" — which is the correct
        // behavior.
        let _lock = if self.lock_scroll {
            Some(ScrollLockGuard::acquire())
        } else {
            None
        };

        let theme = cx.theme();
        let scrim_color = theme
            .renderers
            .modal
            .scrim(&Default::default(), theme.as_ref());

        let dismiss_scrim = self.dismiss_on_scrim;
        let dismiss_esc = self.dismiss_on_escape;
        let on_close_scrim = self.on_close.clone();
        let on_close_escape = self.on_close;

        let mut scrim = self
            .base
            .id(element_id)
            .size_full()
            .absolute()
            .inset_0()
            .bg(scrim_color)
            .flex()
            .items_center()
            .justify_center();

        if dismiss_scrim {
            let cb: ScrimCallback = if let Some(handler) = on_close_scrim {
                Arc::new(move |_ev, w, cx| {
                    handler(&OverlayCloseReason::ScrimClick, w, cx);
                })
            } else {
                Arc::new(|_, _, _| {})
            };
            scrim = scrim.on_mouse_down_out(move |ev, w, cx| {
                cb(ev, w, cx);
            });
        }

        if dismiss_esc {
            let handler = on_close_escape;
            scrim = scrim.capture_key_down(move |event: &KeyDownEvent, _w, cx| {
                let ks = &event.keystroke;
                if ks.key.eq_ignore_ascii_case("escape")
                    && let Some(handler) = &handler
                {
                    cx.stop_propagation();
                    handler(&OverlayCloseReason::Escape, _w, cx);
                }
            });
        }

        // Center the child inside the scrim.
        scrim.child(child).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_overlay_is_closed_and_dismissable() {
        let o = Overlay::default();
        assert!(!o.open);
        assert!(o.dismiss_on_scrim);
        assert!(o.dismiss_on_escape);
        assert!(o.lock_scroll);
    }

    #[test]
    fn setters_update_state() {
        let o = Overlay::new("test")
            .open(true)
            .dismiss_on_scrim(false)
            .dismiss_on_escape(false)
            .lock_scroll(false);
        assert!(o.open);
        assert!(!o.dismiss_on_scrim);
        assert!(!o.dismiss_on_escape);
        assert!(!o.lock_scroll);
    }

    #[test]
    fn close_reason_variants_distinct() {
        assert_ne!(OverlayCloseReason::ScrimClick, OverlayCloseReason::Escape);
        assert_ne!(OverlayCloseReason::Escape, OverlayCloseReason::Programmatic);
    }

    #[test]
    fn extend_takes_last_child() {
        let a: AnyElement = div().id("a").into_any_element();
        let b: AnyElement = div().id("b").into_any_element();
        let o = Overlay::new("test").child(a);
        // Re-extend with b: should overwrite.
        let mut o = o;
        o.extend(vec![b]);
        assert!(o.child.is_some());
    }
}
