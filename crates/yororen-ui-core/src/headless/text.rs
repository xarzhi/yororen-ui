//! Headless `text` — a typed text span. No state.

use gpui::{Div, ElementId, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct TextProps {
    pub id: ElementId,
    pub text: SharedString,
    pub size: Option<gpui::Pixels>,
}

pub fn text(id: impl Into<ElementId>, text: impl Into<SharedString>, _cx: &mut gpui::App) -> TextProps {
    TextProps {
        id: id.into(),
        text: text.into(),
        size: None,
    }
}

impl TextProps {
    pub fn size(mut self, s: impl Into<gpui::Pixels>) -> Self {
        self.size = Some(s.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
