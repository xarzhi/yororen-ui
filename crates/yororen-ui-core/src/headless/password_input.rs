//! Headless `password_input` — text input that masks the value.
//!
//! Reuses `TextInputState` for value + caret; the renderer
//! shows `*` (or a custom `mask_char`) instead of the actual
//! value at paint time.

use std::sync::Arc;

use gpui::{
    App, AppContext, Div, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use super::text_input::TextInputState;

#[derive(Clone)]
pub struct PasswordInputProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub state: Entity<TextInputState>,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<super::text_input::TextChangeCallback>,
    pub on_submit: Option<super::text_input::TextChangeCallback>,
    /// Character to display for each typed letter. Defaults
    /// to `•` (U+2022).
    pub mask_char: char,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn password_input(id: impl Into<ElementId>, cx: &mut App) -> PasswordInputProps {
    PasswordInputProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        state: cx.new(|_| TextInputState::new()),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        on_submit: None,
        mask_char: '•',
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl PasswordInputProps {
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
    pub fn mask_char(mut self, c: char) -> Self {
        self.mask_char = c;
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut Window, &mut App),
    {
        self.on_submit = Some(Arc::new(f));
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
