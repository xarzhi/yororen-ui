//! Headless `spacer` — flexible empty space. No state.

use gpui::{App, Div, ElementId, InteractiveElement, Stateful};
use crate::renderer::RendererContext;

#[derive(Clone, Debug)]
pub struct SpacerProps {
    pub id: ElementId,
}

pub fn spacer(id: impl Into<ElementId>, _cx: &mut gpui::App) -> SpacerProps {
    SpacerProps { id: id.into() }
}

impl SpacerProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the spacer through the registered `SpacerRenderer`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let r = cx
            .renderer_arc::<crate::renderer::markers::Spacer, dyn crate::renderer::spacer::SpacerRenderer>()
            .expect("SpacerRenderer registered");
        r.compose(&self, cx)
    }
}
