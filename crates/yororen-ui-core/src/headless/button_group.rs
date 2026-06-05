//! Headless `button_group` — horizontal/vertical cluster of
//! buttons. No state of its own; the caller composes children.

use gpui::{Div, ElementId, Stateful};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct ButtonGroupProps {
    pub id: ElementId,
    pub orientation: ButtonGroupOrientation,
    pub attached: bool,
}

pub fn button_group(id: impl Into<ElementId>, _cx: &mut gpui::App) -> ButtonGroupProps {
    ButtonGroupProps {
        id: id.into(),
        orientation: ButtonGroupOrientation::default(),
        attached: false,
    }
}

impl ButtonGroupProps {
    pub fn vertical(mut self) -> Self {
        self.orientation = ButtonGroupOrientation::Vertical;
        self
    }
    pub fn attached(mut self, v: bool) -> Self {
        self.attached = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
