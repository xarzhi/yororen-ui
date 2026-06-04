//! \`ModalRoot\` — composite API for modals.

use gpui::{AnyElement, ElementId, IntoElement, RenderOnce, SharedString, div, ParentElement};

use crate::component::modal::Modal;

/// \`ModalRoot\` is the split-API form of \`modal()\`. It
/// mirrors the builder shape but exposes \`.trigger(...)\` and
/// \`.content(...)\` as the primary entry points.
pub struct ModalRoot {
    id: ElementId,
    title: Option<SharedString>,
    content: Option<AnyElement>,
    actions: Option<AnyElement>,
    on_close: Option<Box<dyn Fn(&mut gpui::Window, &mut gpui::App) + Send + Sync>>,
    open: bool,
}

impl ModalRoot {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            content: None,
            actions: None,
            on_close: None,
            open: false,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }

    pub fn on_close<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(&mut gpui::Window, &mut gpui::App) + Send + Sync,
    {
        self.on_close = Some(Box::new(f));
        self
    }

    /// Set the trigger that opens the modal. Optional — modals
    /// can be opened purely via the \`open\` setter.
    pub fn trigger(self, _trigger: impl IntoElement) -> Self {
        // Modals don't have a fixed trigger slot; opening is
        // controlled by \`open\`. We accept \`trigger\` for
        // API symmetry with the other Root types and silently
        // ignore it (the caller can wrap the trigger in a
        // on_click handler that flips the open state).
        self
    }
}

impl RenderOnce for ModalRoot {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let mut m = Modal::new();
        if let Some(t) = self.title {
            m = m.title(t);
        }
        if let Some(c) = self.content {
            m = m.content(c);
        }
        if let Some(a) = self.actions {
            m = m.actions(a);
        }
        if let Some(f) = self.on_close {
            m = m.on_close(f);
        }
        if !self.open {
            m = m.closable(true);
        }
        div().child(m)
    }
}

/// \`ModalTrigger\` — convenience alias for the trigger child.
pub struct ModalTrigger {
    id: ElementId,
    element: AnyElement,
}

impl ModalTrigger {
    pub fn new(id: impl Into<ElementId>, element: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            element: element.into_any_element(),
        }
    }
    pub fn child(&self) -> &AnyElement {
        &self.element
    }
    pub fn id(&self) -> ElementId {
        self.id.clone()
    }
}

/// \`ModalContent\` — convenience alias for the modal body.
pub struct ModalContent {
    id: ElementId,
    element: AnyElement,
}

impl ModalContent {
    pub fn new(id: impl Into<ElementId>, element: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            element: element.into_any_element(),
        }
    }
    pub fn child(&self) -> &AnyElement {
        &self.element
    }
    pub fn id(&self) -> ElementId {
        self.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_root_new_defaults_closed() {
        let r = ModalRoot::new("m1");
        assert!(!r.open);
        assert!(r.title.is_none());
    }

    #[test]
    fn modal_root_chain_setters() {
        let r = ModalRoot::new("m1").open(true).title("Confirm");
        assert!(r.open);
        assert_eq!(r.title.as_ref().map(|s| s.to_string()), Some("Confirm".to_string()));
    }
}
