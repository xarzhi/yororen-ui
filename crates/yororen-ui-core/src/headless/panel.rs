//! Headless `panel` — generic container with optional title. No
//! state of its own; the caller composes the content.

use gpui::{Div, ElementId, Stateful};

#[derive(Clone, Debug)]
pub struct PanelProps {
    pub id: ElementId,
    pub title: Option<String>,
    pub padded: bool,
}

pub fn panel(id: impl Into<ElementId>, _cx: &mut gpui::App) -> PanelProps {
    PanelProps {
        id: id.into(),
        title: None,
        padded: false,
    }
}

impl PanelProps {
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn padded(mut self, v: bool) -> Self {
        self.padded = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
