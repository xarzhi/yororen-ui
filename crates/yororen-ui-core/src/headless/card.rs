//! Headless `card` — generic content surface. No state.

use std::sync::Arc;

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct CardProps {
    pub id: ElementId,
    pub interactive: bool,
    /// `true` if the caller supplied a custom background color
    /// (consumed by `CardRenderer.has_custom_bg`).
    pub has_custom_bg: bool,
}

pub fn card(id: impl Into<ElementId>, _cx: &mut gpui::App) -> CardProps {
    CardProps {
        id: id.into(),
        interactive: false,
        has_custom_bg: false,
    }
}

impl CardProps {
    pub fn interactive(mut self, v: bool) -> Self {
        self.interactive = v;
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the card using the registered `CardRenderer`. Returns
    /// a `Stateful<Div>` with the element id and the renderer-built
    /// bg / border / padding / radius. The caller still chains
    /// `.child(...)` to add content.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::card::CardRenderer;
        use crate::renderer::markers::Card as CardMarker;

        let r: &Arc<dyn CardRenderer> = cx
            .renderer_arc::<CardMarker, dyn CardRenderer>()
            .expect("CardRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
