//! Headless `skeleton` — placeholder shape. No state.

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct SkeletonProps {
    pub id: ElementId,
    pub rounded: bool,
}

pub fn skeleton(id: impl Into<ElementId>, _cx: &mut gpui::App) -> SkeletonProps {
    SkeletonProps {
        id: id.into(),
        rounded: false,
    }
}

impl SkeletonProps {
    pub fn rounded(mut self, v: bool) -> Self {
        self.rounded = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
