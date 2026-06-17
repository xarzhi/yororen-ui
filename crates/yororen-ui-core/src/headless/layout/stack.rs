//! `Stack` — a relative container for Z-axis stacking.
//!
//! Children are responsible for their own positioning via
//! `.absolute()` (typically combined with `top` / `right` /
//! `bottom` / `left` or `inset_0`). `Stack` itself only establishes
//! the `relative` containing block.
//!
//! An `alignment` property that auto-positions children is
//! deferred to Phase 3 (the class-string parser), where
//! `absolute inset-0` etc. can be expressed as class tokens.

use gpui::{
    App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, Stateful, Styled, div,
};

use super::types::{Length, apply_height, apply_width};

pub struct StackProps {
    pub id: ElementId,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub children: Vec<gpui::AnyElement>,
}

pub fn stack(id: impl Into<ElementId>, _cx: &mut App) -> StackProps {
    StackProps {
        id: id.into(),
        width: None,
        height: None,
        children: Vec::new(),
    }
}

impl StackProps {
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
        let mut el: Stateful<Div> = div().id(self.id).relative();
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