//! Headless `empty_state` — icon + title + description.

use std::sync::Arc;

use gpui::{Div, ElementId, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct EmptyStateProps {
    pub id: ElementId,
    pub icon: Option<SharedString>,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub fn empty_state(id: impl Into<ElementId>, _cx: &mut gpui::App) -> EmptyStateProps {
    EmptyStateProps {
        id: id.into(),
        icon: None,
        title: None,
        description: None,
    }
}

impl EmptyStateProps {
    pub fn icon(mut self, i: impl Into<SharedString>) -> Self {
        self.icon = Some(i.into());
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the empty state using the registered
    /// `EmptyStateRenderer`. Returns a `Stateful<Div>` with the
    /// element id and the renderer-built icon / title / body.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::empty_state::EmptyStateRenderer;
        use crate::renderer::markers::EmptyState as EmptyStateMarker;

        let r: &Arc<dyn EmptyStateRenderer> = cx
            .renderer_arc::<EmptyStateMarker, dyn EmptyStateRenderer>()
            .expect("EmptyStateRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
