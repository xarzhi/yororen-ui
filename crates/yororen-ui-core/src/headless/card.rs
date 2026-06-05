//! Headless `card` — generic content surface. No state.

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct CardProps {
    pub id: ElementId,
    pub interactive: bool,
}

pub fn card(id: impl Into<ElementId>, _cx: &mut gpui::App) -> CardProps {
    CardProps {
        id: id.into(),
        interactive: false,
    }
}

impl CardProps {
    pub fn interactive(mut self, v: bool) -> Self {
        self.interactive = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
