//! Headless `focus_ring` — a focus indicator overlay. No state of
//! its own; the renderer reads the bound `FocusHandle` to decide
//! when to draw.

use std::sync::Arc;

use gpui::{Div, ElementId, FocusHandle, InteractiveElement, Stateful};

#[derive(Clone)]
pub struct FocusRingProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    /// `true` if the caller supplied a custom ring color (consumed
    /// by `FocusRingRenderer.has_custom_color`).
    pub has_custom_color: bool,
}

pub fn focus_ring(
    id: impl Into<ElementId>,
    handle: &FocusHandle,
    _cx: &mut gpui::App,
) -> FocusRingProps {
    FocusRingProps {
        id: id.into(),
        focus_handle: handle.clone(),
        has_custom_color: false,
    }
}

impl FocusRingProps {
    pub fn has_custom_color(mut self, v: bool) -> Self {
        self.has_custom_color = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the focus-ring overlay as a `Stateful<Div>`. The
    /// caller still wraps a child element on the returned value
    /// via `.child(...)` — the ring is the border, the child is
    /// the focused content.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::markers::FocusRing as FocusRingMarker;
        use crate::renderer::focus_ring::FocusRingRenderer;
        use crate::renderer::RendererContext;

        let r: &Arc<dyn FocusRingRenderer> = cx
            .renderer_arc::<FocusRingMarker, dyn FocusRingRenderer>()
            .expect("FocusRingRenderer registered");
        r.compose(&self, cx)
    }
}
