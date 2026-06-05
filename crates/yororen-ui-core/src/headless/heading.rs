//! Headless `heading` — a typographic level + text. No visual.

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    #[default]
    H3,
    H4,
    H5,
    H6,
}

#[derive(Clone, Debug)]
pub struct HeadingProps {
    pub id: ElementId,
    pub level: HeadingLevel,
    pub text: String,
}

pub fn heading(
    id: impl Into<ElementId>,
    level: HeadingLevel,
    text: impl Into<String>,
    _cx: &mut gpui::App,
) -> HeadingProps {
    HeadingProps {
        id: id.into(),
        level,
        text: text.into(),
    }
}

impl HeadingProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
