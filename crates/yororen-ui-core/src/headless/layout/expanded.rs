//! `Expanded` — a `div().flex_1()` wrapper with optional children.
//!
//! Used inside `Column` / `Row` to make a child fill the remaining
//! space along the main axis.

use gpui::{
    App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, Stateful, Styled, div,
};

pub struct ExpandedProps {
    pub id: ElementId,
    pub children: Vec<gpui::AnyElement>,
}

pub fn expanded(id: impl Into<ElementId>, _cx: &mut App) -> ExpandedProps {
    ExpandedProps {
        id: id.into(),
        children: Vec::new(),
    }
}

impl ExpandedProps {
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }

    pub fn render(self, _cx: &App) -> Stateful<Div> {
        let mut el: Stateful<Div> = div().id(self.id).flex_1();
        for child in self.children {
            el = el.child(child);
        }
        el
    }
}