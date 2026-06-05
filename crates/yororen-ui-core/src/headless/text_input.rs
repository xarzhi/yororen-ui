//! Headless `text_input` — text state + on_change. No visual.
//!
//! The headless layer only owns the *textual* state machine
//! (value, selection, caret, IME composition) and the standard
//! a11y wiring. Rendering of the value, caret and selection box
//! is the renderer's job (see `yororen-ui-renderer::default`).

use std::sync::Arc;

use gpui::{
    App, Div, ElementId, FocusHandle, InteractiveElement, Stateful, StatefulInteractiveElement,
    Window,
};

pub type TextChangeCallback = Arc<dyn Fn(String, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct TextInputProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<TextChangeCallback>,
    pub on_submit: Option<TextChangeCallback>,
}

pub fn text_input(id: impl Into<ElementId>, cx: &mut App) -> TextInputProps {
    TextInputProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        value: String::new(),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        on_submit: None,
    }
}

impl TextInputProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
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
        F: 'static + Send + Sync + Fn(String, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(String, &mut Window, &mut App),
    {
        self.on_submit = Some(Arc::new(f));
        self
    }

    /// Headless apply. Wires focus tracking and a left-mouse-down
    /// marker (the caller's div decides how to focus on click).
    /// Key dispatch is the renderer's job because caret positioning
    /// depends on the rendered text layout.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id.clone()).track_focus(&self.focus_handle)
    }
}
