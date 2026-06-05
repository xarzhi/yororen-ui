//! Headless `clickable_surface` — generic div with click handler.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, Stateful, StatefulInteractiveElement, Window,
};

pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct ClickableSurfaceProps {
    pub id: ElementId,
    pub on_click: Option<ClickCallback>,
    pub disabled: bool,
}

pub fn clickable_surface(id: impl Into<ElementId>, _cx: &mut App) -> ClickableSurfaceProps {
    ClickableSurfaceProps {
        id: id.into(),
        on_click: None,
        disabled: false,
    }
}

impl ClickableSurfaceProps {
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.on_click = Some(Arc::new(f));
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let s = el.id(self.id.clone());
        if !self.disabled
            && let Some(f) = self.on_click.clone()
        {
            s.on_click(move |ev, window, cx| f(ev, window, cx))
        } else {
            s
        }
    }
}
