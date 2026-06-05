//! Headless `radio_group` — owns a `name` and a current selection
//! index. Keyboard navigation lives in the renderer.

use std::sync::Arc;

use gpui::{App, Div, ElementId, InteractiveElement, SharedString, Stateful};

pub type RadioGroupCallback = Arc<dyn Fn(usize, &mut gpui::Window, &mut gpui::App)>;

#[derive(Clone)]
pub struct RadioGroupProps {
    pub id: ElementId,
    pub name: SharedString,
    pub selected_index: Option<usize>,
    pub on_change: Option<RadioGroupCallback>,
}

pub fn radio_group(id: impl Into<ElementId>, _cx: &mut App) -> RadioGroupProps {
    RadioGroupProps {
        id: id.into(),
        name: "".into(),
        selected_index: None,
        on_change: None,
    }
}

impl RadioGroupProps {
    pub fn name(mut self, n: impl Into<SharedString>) -> Self {
        self.name = n.into();
        self
    }
    pub fn selected(mut self, i: usize) -> Self {
        self.selected_index = Some(i);
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(usize, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
