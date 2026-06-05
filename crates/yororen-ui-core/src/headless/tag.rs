//! Headless `tag` — closeable inline label.

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use std::sync::Arc;

pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct TagProps {
    pub id: ElementId,
    pub label: String,
    pub disabled: bool,
    pub closable: bool,
    pub on_click: Option<ClickCallback>,
    pub on_close: Option<ClickCallback>,
}

pub fn tag(id: impl Into<ElementId>, label: impl Into<String>, _cx: &mut App) -> TagProps {
    TagProps {
        id: id.into(),
        label: label.into(),
        disabled: false,
        closable: false,
        on_click: None,
        on_close: None,
    }
}

impl TagProps {
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn closable(mut self, v: bool) -> Self {
        self.closable = v;
        self
    }
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.on_click = Some(Arc::new(f));
        self
    }
    pub fn on_close<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.on_close = Some(Arc::new(f));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let mut s = el.id(self.id.clone());
        if !self.disabled
            && let Some(f) = self.on_click.clone()
        {
            s = s.on_click(move |ev, window, cx| f(ev, window, cx));
        }
        s
    }
}
