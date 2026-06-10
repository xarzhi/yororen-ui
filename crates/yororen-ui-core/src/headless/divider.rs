//! Headless `divider` — a thin visual line. No state.

use std::sync::Arc;

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct DividerProps {
    pub id: ElementId,
    pub horizontal: bool,
}

pub fn divider(id: impl Into<ElementId>, _cx: &mut gpui::App) -> DividerProps {
    DividerProps {
        id: id.into(),
        horizontal: true,
    }
}

impl DividerProps {
    pub fn vertical(mut self) -> Self {
        self.horizontal = false;
        self
    }

    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the divider using the registered `DividerRenderer`.
    /// Returns a `Div` so the caller can still chain `.w_full()`
    /// / `.h_full()` etc. for the long dimension.
    pub fn render(self, cx: &gpui::App) -> Div {
        use crate::renderer::RendererContext;
        use crate::renderer::divider::DividerRenderer;
        use crate::renderer::markers::Divider as DividerMarker;

        let r: &Arc<dyn DividerRenderer> = cx
            .renderer_arc::<DividerMarker, dyn DividerRenderer>()
            .expect("DividerRenderer registered");
        r.compose(&self, cx)
    }
}
