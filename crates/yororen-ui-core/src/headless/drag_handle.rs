//! Headless `drag_handle` — a span used as a drag source. The
//! headless layer only owns the element id and the value; gpui's
//! `on_drag` is wired in the renderer.

use std::any::Any;

use gpui::{Div, ElementId, InteractiveElement, Stateful};

pub struct DragHandleProps {
    pub id: ElementId,
    pub value: Option<Box<dyn Any + Send + Sync>>,
}

impl Clone for DragHandleProps {
    fn clone(&self) -> Self {
        // `Box<dyn Any + Send + Sync>` is not Clone; cloning a
        // DragHandleProps with a `value` is therefore not supported.
        // The headless API only requires cloning of the id portion,
        // which callers do via `.id.clone()` directly.
        Self {
            id: self.id.clone(),
            value: None,
        }
    }
}

pub fn drag_handle(id: impl Into<ElementId>, _cx: &mut gpui::App) -> DragHandleProps {
    DragHandleProps {
        id: id.into(),
        value: None,
    }
}

impl DragHandleProps {
    pub fn value<T: Any + Send + Sync>(mut self, v: T) -> Self {
        self.value = Some(Box::new(v));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
