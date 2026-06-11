//! Headless `virtual_list` — wraps `gpui::list` with id and a
//! count. The caller passes a render-item closure to the renderer.

use gpui::{App, Div, ElementId, InteractiveElement, Stateful};
use crate::renderer::RendererContext;

#[derive(Clone, Debug)]
pub struct VirtualListProps {
    pub id: ElementId,
    pub item_count: usize,
    pub overdraw_px: Option<f32>,
}

pub fn virtual_list(
    id: impl Into<ElementId>,
    item_count: usize,
    _cx: &mut gpui::App,
) -> VirtualListProps {
    VirtualListProps {
        id: id.into(),
        item_count,
        overdraw_px: None,
    }
}

impl VirtualListProps {
    pub fn overdraw(mut self, px: f32) -> Self {
        self.overdraw_px = Some(px);
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the virtual list container through the registered
    /// `VirtualListRenderer`. Visible items are added as children after
    /// `.render(cx)`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let r = cx
            .renderer_arc::<crate::renderer::markers::VirtualList, dyn crate::renderer::virtual_list::VirtualListRenderer>()
            .expect("VirtualListRenderer registered");
        r.compose(&self, cx)
    }
}
