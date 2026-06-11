//! Headless `shortcut_hint` — small inline `Label` + `KeybindingDisplay`.

use std::sync::Arc;

use gpui::{Div, ElementId, InteractiveElement, SharedString, Stateful};

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

    /// Render the label + keybinding chord using the registered
    /// `ShortcutHintRenderer`. Returns a `Stateful<Div>` with the
    /// element id and the renderer-built row.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::ShortcutHint as ShortcutHintMarker;
        use crate::renderer::shortcut_hint::ShortcutHintRenderer;

        let r: &Arc<dyn ShortcutHintRenderer> = cx
            .renderer_arc::<ShortcutHintMarker, dyn ShortcutHintRenderer>()
            .expect("ShortcutHintRenderer registered");
        r.compose(&self, cx)
    }
}
