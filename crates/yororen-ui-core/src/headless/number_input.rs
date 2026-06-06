//! Headless `number_input` — numeric input with stepper buttons.
//!
//! Reuses `TextInputState` (renderer-minted); the renderer
//! parses the text to `f64` and adds +/- stepper buttons.

use std::sync::Arc;

use gpui::{App, Hsla};

pub type NumberChangeCallback =
    Arc<dyn Fn(f64, &mut gpui::Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct NumberInputProps {
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: f64,
    pub value: f64,
    pub on_change: Option<NumberChangeCallback>,
    pub on_increment: Option<NumberChangeCallback>,
    pub on_decrement: Option<NumberChangeCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn number_input(id: impl Into<gpui::ElementId>) -> NumberInputProps {
    NumberInputProps {
        id: id.into(),
        placeholder: String::new(),
        disabled: false,
        min: None,
        max: None,
        step: 1.0,
        value: 0.0,
        on_change: None,
        on_increment: None,
        on_decrement: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl NumberInputProps {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn min(mut self, v: f64) -> Self {
        self.min = Some(v);
        self
    }
    pub fn max(mut self, v: f64) -> Self {
        self.max = Some(v);
        self
    }
    pub fn step(mut self, v: f64) -> Self {
        self.step = v;
        self
    }
    pub fn value(mut self, v: f64) -> Self {
        self.value = v;
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f64, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_increment<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f64, &mut gpui::Window, &mut App),
    {
        self.on_increment = Some(Arc::new(f));
        self
    }
    pub fn on_decrement<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f64, &mut gpui::Window, &mut App),
    {
        self.on_decrement = Some(Arc::new(f));
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn has_custom_border(mut self, v: bool) -> Self {
        self.has_custom_border = v;
        self
    }
    pub fn has_custom_focus_border(mut self, v: bool) -> Self {
        self.has_custom_focus_border = v;
        self
    }
    pub fn custom_bg(mut self, c: Hsla) -> Self {
        self.custom_bg = Some(c);
        self.has_custom_bg = true;
        self
    }
    pub fn custom_border(mut self, c: Hsla) -> Self {
        self.custom_border = Some(c);
        self.has_custom_border = true;
        self
    }
    pub fn custom_focus_border(mut self, c: Hsla) -> Self {
        self.custom_focus_border = Some(c);
        self.has_custom_focus_border = true;
        self
    }
    pub fn custom_text_color(mut self, c: Hsla) -> Self {
        self.custom_text_color = Some(c);
        self
    }
}
