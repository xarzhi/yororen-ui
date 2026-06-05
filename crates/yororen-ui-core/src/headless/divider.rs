//! Headless `divider` — a thin visual line. No state.

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct DividerProps {
    pub id: ElementId,
    pub horizontal: bool,
}

pub fn divider(id: impl Into<ElementId>, _cx: &mut gpui::App) -> DividerProps {
    DividerProps {
        id: id.into(),
        horizontal: true,
    }
}

impl DividerProps {
    pub fn vertical(mut self) -> Self {
        self.horizontal = false;
        self
    }

    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
