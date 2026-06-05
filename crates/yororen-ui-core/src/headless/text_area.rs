//! Headless `text_area` — multi-line text input.
//!
//! Wraps the same `TextInputState` as `text_input`; the only
//! behavioural difference is that `Enter` inserts a newline
//! instead of firing `on_submit`.

use std::sync::Arc;

use gpui::{
    App, AppContext, Div, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use super::text_input::TextInputState;

/// `TextArea` is a multi-line text input. It reuses
/// `TextInputState` for value + caret.
#[derive(Clone)]
pub struct TextAreaProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub state: Entity<TextInputState>,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<super::text_input::TextChangeCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn text_area(id: impl Into<ElementId>, cx: &mut App) -> TextAreaProps {
    TextAreaProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        state: cx.new(|_| TextInputState::new()),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl TextAreaProps {
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
    pub fn max_length(mut self, v: usize) -> Self {
        self.max_length = Some(v);
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
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
