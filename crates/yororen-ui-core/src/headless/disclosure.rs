//! Headless `disclosure` — collapsible content with an `open` flag.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, InteractiveElement, Stateful, StatefulInteractiveElement,
    Window,
};

pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct DisclosureProps {
    pub id: ElementId,
    pub title: String,
    pub open: bool,
    pub disabled: bool,
    pub on_toggle: Option<ClickCallback>,
}

pub fn disclosure(
    id: impl Into<ElementId>,
    title: impl Into<String>,
    _cx: &mut App,
) -> DisclosureProps {
    DisclosureProps {
        id: id.into(),
        title: title.into(),
        open: false,
        disabled: false,
        on_toggle: None,
    }
}

impl DisclosureProps {
    pub fn open(mut self, v: bool) -> Self {
        self.open = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(f));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let mut s = el.id(self.id.clone());
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            s = s.on_click(move |ev, window, cx| f(ev, window, cx));
        }
        s
    }

    /// Render the disclosure trigger using the registered
    /// `DisclosureRenderer`. Returns a `Stateful<Div>` with the
    /// element id and on_toggle wired by `apply`.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::disclosure::DisclosureRenderer;
        use crate::renderer::markers::Disclosure as DisclosureMarker;

        let r: &Arc<dyn DisclosureRenderer> = cx
            .renderer_arc::<DisclosureMarker, dyn DisclosureRenderer>()
            .expect("DisclosureRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
