//! Layout `spacer` — flexible empty space. No state, no renderer.
//!
//! This is the layout-module spacer: a simpler `div().flex_1().id(id)`
//! with no renderer lookup overhead. The existing
//! [`crate::headless::spacer::SpacerProps`] (which goes through a
//! `SpacerRenderer` trait) remains available for backward
//! compatibility — the two types are distinct and distinguished
//! by the `layout::` path prefix.

use gpui::{App, Div, ElementId, InteractiveElement, Stateful, Styled, div};

#[derive(Clone, Debug)]
pub struct SpacerProps {
    pub id: ElementId,
}

pub fn spacer(id: impl Into<ElementId>, _cx: &mut App) -> SpacerProps {
    SpacerProps { id: id.into() }
}

impl SpacerProps {
    pub fn render(self, _cx: &App) -> Stateful<Div> {
        div().id(self.id).flex_1()
    }
}