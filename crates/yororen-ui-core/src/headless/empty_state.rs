//! Headless `empty_state` — icon + title + description.

use gpui::{Div, ElementId, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct EmptyStateProps {
    pub id: ElementId,
    pub icon: Option<SharedString>,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub fn empty_state(id: impl Into<ElementId>, _cx: &mut gpui::App) -> EmptyStateProps {
    EmptyStateProps {
        id: id.into(),
        icon: None,
        title: None,
        description: None,
    }
}

impl EmptyStateProps {
    pub fn icon(mut self, i: impl Into<SharedString>) -> Self {
        self.icon = Some(i.into());
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
