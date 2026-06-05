//! Headless `focus_ring` — a focus indicator overlay. No state of
//! its own; the renderer reads the bound `FocusHandle` to decide
//! when to draw.

use gpui::{Div, ElementId, FocusHandle, InteractiveElement, Stateful};

#[derive(Clone)]
pub struct FocusRingProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
}

pub fn focus_ring(
    id: impl Into<ElementId>,
    handle: &FocusHandle,
    _cx: &mut gpui::App,
) -> FocusRingProps {
    FocusRingProps {
        id: id.into(),
        focus_handle: handle.clone(),
    }
}

impl FocusRingProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
