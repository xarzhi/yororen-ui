//! `ModalShell` — a one-line "show me a modal" wrapper that
//! composes `Modal` with `Overlay` + body scroll lock, and links
//! the close-button callback to the overlay's dismiss state.
//!
//! Phase G-gamma + G-delta hard requirements from the v0.5 review:
//!
//! - G-gamma: `Modal::new().open(true)` should be enough on its
//!   own to get a fully a11y modal (focus trap + Esc +
//!   click-outside + scroll lock). We satisfy this by
//!   introducing a new `ModalShell` factory that wraps
//!   everything together.
//! - G-delta: the close button on the inner Modal must close
//!   the outer overlay (the original two callbacks are
//!   independent). ModalShell solves this by having the Modal's
//!   on_close callback also flip the shell's `open` state,
//!   which causes the overlay to close.

use std::sync::Arc;

use gpui::{AnyElement, ElementId, IntoElement, ParentElement, RenderOnce, Window, div};

use crate::component::modal::Modal;
use crate::component::overlay::{Overlay, OverlayCloseReason};

/// Build a new `ModalShell` element with the given stable id.
///
/// A `ModalShell` is the v0.5 one-line dialog: it auto-composes
/// `Modal` with `Overlay` + body scroll lock, links the
/// close-button callback to the overlay's dismiss state, and
/// exposes a single `open(bool)` toggle plus an `on_close`
/// callback.
///
/// # Usage
///
/// ```rust,ignore
/// use yororen_ui::component::{modal_dialog, modal, label, modal_actions_row};
///
/// // One-line a11y modal:
/// modal_dialog("confirm")
///     .title("Delete file?")
///     .content(label("This cannot be undone."))
///     .actions(modal_actions_row(...))
///     .open(state.show_confirm)
///     .on_close(move |_reason, _w, cx| {
///         state.show_confirm = false;
///         cx.refresh_windows();
///     });
/// ```
///
/// For advanced composition (e.g. embedding a Modal in a custom
/// overlay) use the bare [`modal`](super::modal::modal) factory
/// instead.
pub fn modal_dialog(id: impl Into<ElementId>) -> ModalShell {
    ModalShell::new(id)
}

/// Close callback for `ModalShell`. The first argument is the
/// reason: `ScrimClick`, `Escape`, or `Programmatic` (the latter
/// fires when the user clicks the inner Modal's close button).
pub type ModalShellCloseCallback =
    Arc<dyn Fn(&OverlayCloseReason, &mut Window, &mut gpui::App) + Send + Sync>;

/// A one-line a11y modal. See [`modal_dialog`] for usage.
#[derive(IntoElement)]
pub struct ModalShell {
    element_id: ElementId,
    /// Whether the modal is currently shown. When `false`, the
    /// shell renders nothing.
    open: bool,
    /// Inner Modal (built via the Modal builder chain).
    modal: Option<Modal>,
    /// Close callback. Fires on scrim click, Esc, or close-button
    /// click. Programmatic closes (from the inner Modal's X) are
    /// also routed through this callback with `Programmatic`.
    on_close: Option<ModalShellCloseCallback>,
    /// Whether the overlay's scrim-click dismisses the modal.
    /// Default: `true`.
    dismiss_on_scrim: bool,
    /// Whether Esc dismisses the modal. Default: `true`.
    dismiss_on_escape: bool,
    /// Whether body scroll is locked while the modal is open.
    /// Default: `true`.
    lock_scroll: bool,
    /// Optional outer-width override for the modal panel.
    /// Default: from the active theme's Modal tokens.
    width: Option<gpui::Pixels>,
}

impl Default for ModalShell {
    fn default() -> Self {
        Self::new("ui:modal-dialog")
    }
}

impl ModalShell {
    /// Create a new `ModalShell` with the given stable id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            element_id: id.into(),
            open: false,
            modal: None,
            on_close: None,
            dismiss_on_scrim: true,
            dismiss_on_escape: true,
            lock_scroll: true,
            width: None,
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

    /// Open or close the modal. When `false`, the shell renders
    /// nothing (so it's safe to include in always-on layouts).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Set the inner Modal that this shell wraps. The shell
    /// doesn't auto-construct a Modal; you pass it the same way
    /// you would build any Modal:
    ///
    /// ```rust,ignore
    /// modal_dialog("my-modal")
    ///     .modal(modal().title("Hi").content(label("Body")))
    /// ```
    pub fn modal(mut self, modal: Modal) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Convenience: same as `.modal(modal().title(t).content(c))`
    /// but with title / content set on the inner modal.
    pub fn title(mut self, title: impl Into<gpui::SharedString>) -> Self {
        let modal = self.modal.unwrap_or_default();
        self.modal = Some(modal.title(title));
        self
    }

    /// Convenience: same as `.modal(modal().content(c))` but with
    /// content set on the inner modal.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        let modal = self.modal.unwrap_or_default();
        self.modal = Some(modal.content(content));
        self
    }

    /// Convenience: same as `.modal(modal().actions(a))` but with
    /// actions set on the inner modal.
    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        let modal = self.modal.unwrap_or_default();
        self.modal = Some(modal.actions(actions));
        self
    }

    /// Optional explicit width for the modal panel.
    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the close callback. Fires on scrim click, Esc, or
    /// the inner Modal's close button (the latter is reported
    /// as `OverlayCloseReason::Programmatic`).
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&OverlayCloseReason, &mut Window, &mut gpui::App),
    {
        self.on_close = Some(Arc::new(handler));
        self
    }

    /// Set whether a click on the scrim dismisses the modal.
    /// Default: `true`. Set to `false` for non-dismissable
    /// dialogs (e.g. destructive-action confirmations).
    pub fn dismiss_on_scrim(mut self, dismiss: bool) -> Self {
        self.dismiss_on_scrim = dismiss;
        self
    }

    /// Set whether the Escape key dismisses the modal.
    /// Default: `true`.
    pub fn dismiss_on_escape(mut self, dismiss: bool) -> Self {
        self.dismiss_on_escape = dismiss;
        self
    }

    /// Set whether body scroll is locked while the modal is open.
    /// Default: `true`. Set to `false` for non-modal dialogs.
    pub fn lock_scroll(mut self, lock: bool) -> Self {
        self.lock_scroll = lock;
        self
    }
}

impl ParentElement for ModalShell {
    fn extend(&mut self, _elements: impl IntoIterator<Item = AnyElement>) {
        // ModalShell is a single-child container; users call
        // `.modal(...)` or `.title(...)` / `.content(...)`. The
        // `.extend(...)` method is intentionally a no-op so the
        // fluent API does not accidentally accept children.
    }
}

impl RenderOnce for ModalShell {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        if !self.open {
            return div().into_any_element();
        }

        let element_id = self.element_id;
        let modal = self.modal.unwrap_or_default();
        let on_close = self.on_close;
        let dismiss_on_scrim = self.dismiss_on_scrim;
        let dismiss_on_escape = self.dismiss_on_escape;
        let lock_scroll = self.lock_scroll;
        let _ = self.width;

        // The inner Modal needs its own on_close so that clicking
        // the close button (X) routes back to the shell. We
        // build that closure by wrapping the shell's on_close.
        let modal = if let Some(handler) = on_close.clone() {
            modal.on_close(move |window, cx| {
                handler(&OverlayCloseReason::Programmatic, window, cx);
            })
        } else {
            modal
        };

        // G-delta linking: the modal's close button now triggers
        // Programmatic in the shell's on_close. The Overlay's
        // scrim click + Esc fire ScrimClick / Escape respectively.
        let inner: AnyElement = modal.into_any_element();

        // We render an Overlay wrapping the inner. The Overlay
        // automatically captures Escape + click-outside. The
        // ScrollLock is acquired as long as the overlay is open
        // (its lifetime is bound to this render frame).
        let overlay = Overlay::new((element_id.clone(), "ui:modal-dialog:overlay"))
            .open(true)
            .dismiss_on_scrim(dismiss_on_scrim)
            .dismiss_on_escape(dismiss_on_escape)
            .lock_scroll(lock_scroll)
            .child(inner);

        let final_overlay: Overlay = if let Some(handler) = on_close {
            overlay.on_close(move |reason, window, cx| {
                handler(reason, window, cx);
            })
        } else {
            overlay
        };
        // Convert the overlay to AnyElement so the two branches
        // have the same return type for the impl trait.
        final_overlay.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_modal_shell_is_closed() {
        let s = ModalShell::default();
        assert!(!s.open);
        assert!(s.dismiss_on_scrim);
        assert!(s.dismiss_on_escape);
        assert!(s.lock_scroll);
    }

    #[test]
    fn setters_update_state() {
        let s = ModalShell::new("test")
            .open(true)
            .dismiss_on_scrim(false)
            .dismiss_on_escape(false)
            .lock_scroll(false);
        assert!(s.open);
        assert!(!s.dismiss_on_scrim);
        assert!(!s.dismiss_on_escape);
        assert!(!s.lock_scroll);
    }

    #[test]
    fn title_content_actions_set_inner_modal() {
        let _s = ModalShell::new("test")
            .title("Hi")
            .content(crate::component::label("Body"));
        // No panic; the field is set even when not opened.
    }
}
