//! Headless `modal` — owns `open` + focus-trap config + close
//! reasons. The renderer draws a scrim, a centered dialog, and a
//! focus trap, and routes `Escape` / scrim-click to `on_close`.

use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, FocusHandle, InteractiveElement, Stateful};

/// Reason a modal was closed. Forwarded to the caller's
/// `on_close` so it can branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalCloseReason {
    Escape,
    ScrimClick,
    Programmatic,
}

pub type ModalCloseCallback =
    Arc<dyn Fn(ModalCloseReason, &mut gpui::Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct ModalState {
    pub open: bool,
    pub dismiss_on_escape: bool,
    pub dismiss_on_scrim: bool,
    /// Focus handle for the *initial* focus when the modal opens.
    /// The renderer traps focus inside the dialog.
    pub initial_focus: Option<FocusHandle>,
    /// Optional label for the scrim (read by `aria-modal` etc.).
    pub title: Option<String>,
    on_close: Option<ModalCloseCallback>,
}

impl ModalState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            open: false,
            dismiss_on_escape: true,
            dismiss_on_scrim: true,
            initial_focus: None,
            title: None,
            on_close: None,
        })
    }

    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn set_dismiss_on_escape(&mut self, v: bool) {
        self.dismiss_on_escape = v;
    }
    pub fn set_dismiss_on_scrim(&mut self, v: bool) {
        self.dismiss_on_scrim = v;
    }
    pub fn set_initial_focus(&mut self, h: FocusHandle) {
        self.initial_focus = Some(h);
    }
    pub fn set_title(&mut self, t: impl Into<String>) {
        self.title = Some(t.into());
    }
    pub fn set_on_close<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(ModalCloseReason, &mut gpui::Window, &mut App),
    {
        self.on_close = Some(Arc::new(f));
    }
    pub fn invoke_close(&self, reason: ModalCloseReason, window: &mut gpui::Window, cx: &mut App) {
        if let Some(f) = &self.on_close {
            f(reason, window, cx);
        }
    }
}

#[derive(Clone)]
pub struct ModalProps {
    pub id: ElementId,
    pub state: Entity<ModalState>,
}

pub fn modal(id: impl Into<ElementId>, state: Entity<ModalState>) -> ModalProps {
    ModalProps {
        id: id.into(),
        state,
    }
}

impl ModalProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
