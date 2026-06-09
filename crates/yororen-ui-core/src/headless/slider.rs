//! Headless `slider` — value in [min, max] with on_change.

use std::sync::Arc;

use gpui::{App, Div, ElementId, InteractiveElement, Stateful, Window};

pub type SliderCallback = Arc<dyn Fn(f32, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct SliderProps {
    pub id: ElementId,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub disabled: bool,
    pub on_change: Option<SliderCallback>,
}

pub fn slider(id: impl Into<ElementId>, _cx: &mut App) -> SliderProps {
    SliderProps {
        id: id.into(),
        value: 0.0,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        disabled: false,
        on_change: None,
    }
}

impl SliderProps {
    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn step(mut self, s: f32) -> Self {
        self.step = s;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f32, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
