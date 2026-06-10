//! Headless `split_button` — main action + attached dropdown
//! trigger. State is delegated to the caller; the headless layer
//! only owns the two click handlers.

use std::sync::Arc;

use gpui::{App, ClickEvent, Div, ElementId, InteractiveElement, Stateful, Window};

pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct SplitButtonProps {
    pub id: ElementId,
    pub primary: ClickCallback,
    pub secondary: Option<ClickCallback>,
    pub disabled: bool,
}

pub fn split_button(
    id: impl Into<ElementId>,
    primary: impl 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    _cx: &mut App,
) -> SplitButtonProps {
    SplitButtonProps {
        id: id.into(),
        primary: Arc::new(primary),
        secondary: None,
        disabled: false,
    }
}

impl SplitButtonProps {
    pub fn on_secondary<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.secondary = Some(Arc::new(f));
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the split button using the registered
    /// `SplitButtonRenderer`. Returns a `Stateful<Div>` with the
    /// element id. The caller adds the children (primary + chevron
    /// triggers).
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::split_button::SplitButtonRenderer;
        use crate::renderer::markers::SplitButton as SplitButtonMarker;

        let r: &Arc<dyn SplitButtonRenderer> = cx
            .renderer_arc::<SplitButtonMarker, dyn SplitButtonRenderer>()
            .expect("SplitButtonRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
