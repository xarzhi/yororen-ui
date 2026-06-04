//! \`TooltipRoot\` — composite API for tooltips.

use gpui::{AnyElement, ElementId, IntoElement, RenderOnce, div, ParentElement};

/// \`TooltipRoot\` is the split-API form of \`tooltip()\`. It
/// mirrors the builder shape but exposes \`.trigger(...)\` and
/// \`.content(...)\` as the primary entry points.
pub struct TooltipRoot {
    id: ElementId,
    trigger: Option<AnyElement>,
    content: Option<AnyElement>,
}

impl TooltipRoot {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
        }
    }

    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
}

impl RenderOnce for TooltipRoot {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        // Tooltips are reactive — they appear on hover. The
        // Root/Trigger/Content split is mostly a documentation
        // aid here: the user is expected to pass a single
        // element to the underlying `tooltip()` builder, which
        // we approximate by wrapping `trigger + content` in a
        // single div. Real tooltip wiring is in the
        // component::tooltip module.
        let trigger = self.trigger.unwrap_or_else(|| div().into_any_element());
        let content = self.content.unwrap_or_else(|| div().into_any_element());
        div().child(trigger).child(content)
    }
}

/// \`TooltipTrigger\` — convenience alias.
pub struct TooltipTrigger {
    id: ElementId,
    element: AnyElement,
}

impl TooltipTrigger {
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

/// \`TooltipContent\` — convenience alias.
pub struct TooltipContent {
    id: ElementId,
    element: AnyElement,
}

impl TooltipContent {
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
    fn tooltip_root_defaults() {
        let r = TooltipRoot::new("t1");
        assert!(r.trigger.is_none());
        assert!(r.content.is_none());
    }
}
