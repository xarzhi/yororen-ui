//! Headless `number_input` — numeric input with stepper buttons.
//!
//! Reuses `TextInputState` (raw text) and parses to `f64` on
//! demand. The renderer adds +/- stepper buttons that call
//! `on_increment` / `on_decrement`.

use std::sync::Arc;

use gpui::{
    App, AppContext, Div, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use super::text_input::TextInputState;

pub type NumberChangeCallback =
    Arc<dyn Fn(f64, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct NumberInputProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub state: Entity<TextInputState>,
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

pub fn number_input(id: impl Into<ElementId>, cx: &mut App) -> NumberInputProps {
    NumberInputProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        state: cx.new(|_| TextInputState::new()),
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
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn state(&self) -> &Entity<TextInputState> {
        &self.state
    }
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
        F: 'static + Send + Sync + Fn(f64, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_increment<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f64, &mut Window, &mut App),
    {
        self.on_increment = Some(Arc::new(f));
        self
    }
    pub fn on_decrement<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f64, &mut Window, &mut App),
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
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id).track_focus(&self.focus_handle)
    }
}
