//! Headless `spacer` — flexible empty space. No state.

use gpui::{Div, ElementId, Stateful};

#[derive(Clone, Debug)]
pub struct SpacerProps {
    pub id: ElementId,
}

pub fn spacer(id: impl Into<ElementId>, _cx: &mut gpui::App) -> SpacerProps {
    SpacerProps { id: id.into() }
}

impl SpacerProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
