//! Headless `form` — owns field values + validation + submit.

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{App, Div, ElementId, InteractiveElement, SharedString, Stateful};

pub type FormSubmitCallback =
    Arc<dyn Fn(HashMap<SharedString, String>, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct FormProps {
    pub id: ElementId,
    pub values: HashMap<SharedString, String>,
    pub errors: HashMap<SharedString, String>,
    pub on_submit: Option<FormSubmitCallback>,
}

pub fn form(id: impl Into<ElementId>, _cx: &mut App) -> FormProps {
    FormProps {
        id: id.into(),
        values: HashMap::new(),
        errors: HashMap::new(),
        on_submit: None,
    }
}

impl FormProps {
    pub fn value(mut self, field: impl Into<SharedString>, v: impl Into<String>) -> Self {
        self.values.insert(field.into(), v.into());
        self
    }
    pub fn error(mut self, field: impl Into<SharedString>, e: impl Into<String>) -> Self {
        self.errors.insert(field.into(), e.into());
        self
    }
    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(HashMap<SharedString, String>, &mut gpui::Window, &mut App),
    {
        self.on_submit = Some(Arc::new(f));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
