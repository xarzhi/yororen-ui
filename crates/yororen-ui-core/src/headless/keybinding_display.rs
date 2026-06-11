//! Headless `keybinding_display` — renders a chord like `⌘ S`.

use std::sync::Arc;

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

    /// Render the chord using the registered `KeybindingDisplayRenderer`.
    /// Returns a `Stateful<Div>` with the element id and the
    /// renderer-built row of key caps.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::keybinding_display::KeybindingDisplayRenderer;
        use crate::renderer::markers::KeybindingDisplay as KeybindingDisplayMarker;

        let r: &Arc<dyn KeybindingDisplayRenderer> = cx
            .renderer_arc::<KeybindingDisplayMarker, dyn KeybindingDisplayRenderer>()
            .expect("KeybindingDisplayRenderer registered");
        r.compose(&self, cx)
    }
}
