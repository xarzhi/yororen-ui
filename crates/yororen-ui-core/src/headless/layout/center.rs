//! `Center` — a flex container with `items_center` + `justify_center`.

use gpui::{
    App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, Stateful, Styled, div,
};

use super::types::{Length, apply_height, apply_width};

pub struct CenterProps {
    pub id: ElementId,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub children: Vec<gpui::AnyElement>,
}

pub fn center(id: impl Into<ElementId>, _cx: &mut App) -> CenterProps {
    CenterProps {
        id: id.into(),
        width: None,
        height: None,
        children: Vec::new(),
    }
}

impl CenterProps {
    pub fn w(mut self, w: Length) -> Self {
        self.width = Some(w);
        self
    }
    pub fn w_full(mut self) -> Self {
        self.width = Some(Length::Full);
        self
    }
    pub fn h(mut self, h: Length) -> Self {
        self.height = Some(h);
        self
    }
    pub fn h_full(mut self) -> Self {
        self.height = Some(Length::Full);
        self
    }

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
        let mut el: Stateful<Div> = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center();
        if let Some(w) = self.width {
            el = apply_width(el, w);
        }
        if let Some(h) = self.height {
            el = apply_height(el, h);
        }
        for child in self.children {
            el = el.child(child);
        }
        el
    }
}