//! Headless `list_item` — a single row in a list. Pure data
//! carrier; visual lives in the renderer.

use gpui::{Div, ElementId, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct ListItemProps {
    pub id: ElementId,
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub leading_icon: Option<SharedString>,
    pub trailing_icon: Option<SharedString>,
    pub selected: bool,
    pub disabled: bool,
}

pub fn list_item(id: impl Into<ElementId>, title: impl Into<SharedString>, _cx: &mut gpui::App) -> ListItemProps {
    ListItemProps {
        id: id.into(),
        title: title.into(),
        description: None,
        leading_icon: None,
        trailing_icon: None,
        selected: false,
        disabled: false,
    }
}

impl ListItemProps {
    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn leading_icon(mut self, i: impl Into<SharedString>) -> Self {
        self.leading_icon = Some(i.into());
        self
    }
    pub fn trailing_icon(mut self, i: impl Into<SharedString>) -> Self {
        self.trailing_icon = Some(i.into());
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
