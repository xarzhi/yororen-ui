//! Headless `split_button` — main action + attached dropdown
//! trigger. State is delegated to the caller; the headless layer
//! only owns the two click handlers.

use std::sync::Arc;

use gpui::{App, ClickEvent, Div, ElementId, Stateful, StatefulInteractiveElement, Window};

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
    primary: impl 'static + Fn(&ClickEvent, &mut Window, &mut App),
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
        F: 'static + Fn(&ClickEvent, &mut Window, &mut App),
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
}
