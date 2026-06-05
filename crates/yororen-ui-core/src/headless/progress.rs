//! Headless `progress` — a value in [0, max] with optional label.

use gpui::{App, Div, ElementId, Stateful};

pub type ProgressCallback = std::sync::Arc<dyn Fn(f32, &mut gpui::Window, &mut gpui::App)>;

#[derive(Clone, Debug)]
pub struct ProgressBarProps {
    pub id: ElementId,
    pub value: f32,
    pub max: f32,
    pub label: Option<String>,
}

pub fn progress(id: impl Into<ElementId>, _cx: &mut App) -> ProgressBarProps {
    ProgressBarProps {
        id: id.into(),
        value: 0.0,
        max: 1.0,
        label: None,
    }
}

impl ProgressBarProps {
    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }
    pub fn max(mut self, m: f32) -> Self {
        self.max = m;
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
