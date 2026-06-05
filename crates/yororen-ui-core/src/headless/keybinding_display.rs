//! Headless `keybinding_display` — renders a chord like `⌘ S`.

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct KeybindingDisplayProps {
    pub id: ElementId,
    pub keys: Vec<String>,
}

pub fn keybinding_display(
    id: impl Into<ElementId>,
    keys: impl IntoIterator<Item = impl Into<String>>,
    _cx: &mut gpui::App,
) -> KeybindingDisplayProps {
    KeybindingDisplayProps {
        id: id.into(),
        keys: keys.into_iter().map(Into::into).collect(),
    }
}

impl KeybindingDisplayProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
