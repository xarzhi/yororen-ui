//! Headless `shortcut_hint` — small inline `Label` + `KeybindingDisplay`.

use gpui::{Div, ElementId, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct ShortcutHintProps {
    pub id: ElementId,
    pub label: SharedString,
    pub keys: Vec<String>,
}

pub fn shortcut_hint(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    keys: impl IntoIterator<Item = impl Into<String>>,
    _cx: &mut gpui::App,
) -> ShortcutHintProps {
    ShortcutHintProps {
        id: id.into(),
        label: label.into(),
        keys: keys.into_iter().map(Into::into).collect(),
    }
}

impl ShortcutHintProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
