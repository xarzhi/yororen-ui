//! \`PopoverRoot\` — composite API for popovers.
//!
//! Split form of the existing \`popover()\` builder. Equivalent
//! behavior; pick whichever reads better at the call site.

use gpui::{AnyElement, ElementId, IntoElement, ParentElement, RenderOnce, div, px};

use crate::component::popover::{Popover, PopoverPlacement};

/// \`PopoverRoot\` is the split-API form of \`popover()\`. It
/// mirrors the builder shape but exposes \`.trigger(...)\` and
/// \`.content(...)\` as the primary entry points.
pub struct PopoverRoot {
    id: ElementId,
    open: bool,
    placement: PopoverPlacement,
    width: Option<gpui::Pixels>,
    on_close: Option<Box<dyn Fn(&mut gpui::Window, &mut gpui::App) + Send + Sync>>,
    trigger: Option<AnyElement>,
    content: Option<AnyElement>,
}

impl PopoverRoot {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            open: false,
            placement: PopoverPlacement::BottomStart,
            width: None,
            on_close: None,
            trigger: None,
            content: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn width(mut self, width: impl Into<gpui::Pixels>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_close<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(&mut gpui::Window, &mut gpui::App) + Send + Sync,
    {
        self.on_close = Some(Box::new(f));
        self
    }

    /// Set the trigger element. The element is rendered at the
    /// popover anchor. The popover opens on click.
    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    /// Set the popover body. The body is rendered next to the
    /// trigger (or above / below, per placement) when \`open\` is
    /// true.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
}

impl RenderOnce for PopoverRoot {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let mut p = Popover::new(self.id)
            .open(self.open)
            .placement(self.placement);
        if let Some(w) = self.width {
            p = p.width(w);
        }
        if let Some(t) = self.trigger {
            p = p.trigger(t);
        }
        if let Some(c) = self.content {
            p = p.content(c);
        }
        if let Some(f) = self.on_close {
            p = p.on_close(f);
        }
        div().child(p)
    }
}

/// \`PopoverTrigger\` — convenience alias for the trigger child.
/// Equivalent to passing the element to \`PopoverRoot::trigger\`.
pub struct PopoverTrigger {
    id: ElementId,
    element: AnyElement,
}

impl PopoverTrigger {
    pub fn new(id: impl Into<ElementId>, element: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            element: element.into_any_element(),
        }
    }
}

/// \`PopoverContent\` — convenience alias for the popover body.
pub struct PopoverContent {
    id: ElementId,
    element: AnyElement,
    width: Option<gpui::Pixels>,
}

impl PopoverContent {
    pub fn new(id: impl Into<ElementId>, element: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            element: element.into_any_element(),
            width: None,
        }
    }
    pub fn width(mut self, w: impl Into<gpui::Pixels>) -> Self {
        self.width = Some(w.into());
        self
    }
    pub fn child(&self) -> &AnyElement {
        &self.element
    }
    pub fn id(&self) -> ElementId {
        self.id.clone()
    }
    pub fn width_value(&self) -> Option<gpui::Pixels> {
        self.width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popover_root_new_defaults_closed() {
        let r = PopoverRoot::new("p1");
        assert!(!r.open);
    }

    #[test]
    fn popover_root_chain_setters() {
        let r = PopoverRoot::new("p1")
            .open(true)
            .width(px(200.0))
            .placement(PopoverPlacement::BottomEnd);
        assert!(r.open);
        assert!(r.width.is_some());
    }

    #[test]
    fn popover_content_default_has_no_width() {
        let c = PopoverContent::new("pc", div());
        assert!(c.width_value().is_none());
    }
}
