//! Headless `panel` — generic container with optional title. No
//! state of its own; the caller composes the content.

use std::sync::Arc;

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct PanelProps {
    pub id: ElementId,
    pub title: Option<String>,
    pub padded: bool,
    /// `true` if the caller supplied a custom background color
    /// (consumed by `PanelRenderer.has_custom_bg`).
    pub has_custom_bg: bool,
    /// `true` if the caller supplied a custom border (consumed
    /// by `PanelRenderer.has_custom_border`).
    pub has_custom_border: bool,
    /// `true` if the caller supplied a custom padding (consumed
    /// by `PanelRenderer.has_custom_padding`).
    pub has_custom_padding: bool,
}

pub fn panel(id: impl Into<ElementId>, _cx: &mut gpui::App) -> PanelProps {
    PanelProps {
        id: id.into(),
        title: None,
        padded: false,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_padding: false,
    }
}

impl PanelProps {
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn padded(mut self, v: bool) -> Self {
        self.padded = v;
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn has_custom_border(mut self, v: bool) -> Self {
        self.has_custom_border = v;
        self
    }
    pub fn has_custom_padding(mut self, v: bool) -> Self {
        self.has_custom_padding = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the panel using the registered `PanelRenderer`.
    /// Returns a `Stateful<Div>` with the element id and the
    /// renderer-built bg / border / padding / radius.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::panel::PanelRenderer;
        use crate::renderer::markers::Panel as PanelMarker;

        let r: &Arc<dyn PanelRenderer> = cx
            .renderer_arc::<PanelMarker, dyn PanelRenderer>()
            .expect("PanelRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
