//! Headless `avatar` — image or initials in a circle.

use gpui::{Div, ElementId, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct AvatarProps {
    pub id: ElementId,
    pub src: Option<SharedString>,
    pub initials: Option<String>,
    pub name: Option<SharedString>,
    pub size: Option<gpui::Pixels>,
}

pub fn avatar(id: impl Into<ElementId>, _cx: &mut gpui::App) -> AvatarProps {
    AvatarProps {
        id: id.into(),
        src: None,
        initials: None,
        name: None,
        size: None,
    }
}

impl AvatarProps {
    pub fn src(mut self, s: impl Into<SharedString>) -> Self {
        self.src = Some(s.into());
        self
    }
    pub fn initials(mut self, i: impl Into<String>) -> Self {
        self.initials = Some(i.into());
        self
    }
    pub fn name(mut self, n: impl Into<SharedString>) -> Self {
        self.name = Some(n.into());
        self
    }
    pub fn size(mut self, s: impl Into<gpui::Pixels>) -> Self {
        self.size = Some(s.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
