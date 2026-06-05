//! Headless `text_input` — text state + on_change. No visual.
//!
//! The headless layer owns the *textual* state machine
//! (value, caret position, IME composition) and exposes it as a
//! `gpui::Entity<TextInputState>` so the renderer (and the
//! key-event handler) can read/mutate the value without copying
//! strings through closure captures on every keystroke.
//!
//! Rendering of the value, caret and selection box is the
//! renderer's job (see `yororen-ui-default-renderer::text_input`).

use std::sync::Arc;

use gpui::{
    App, AppContext, Div, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

pub type TextChangeCallback = Arc<dyn Fn(&str, &mut Window, &mut App) + Send + Sync>;

/// Text input state — the value + caret position. Held in a
/// `gpui::Entity` so the renderer's key handlers can mutate it
/// without copying strings.
#[derive(Clone, Debug)]
pub struct TextInputState {
    pub value: String,
    /// Caret position, in bytes. `0..=value.len()`.
    pub caret: usize,
}

impl TextInputState {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            caret: 0,
        }
    }

    pub fn with_value(value: impl Into<String>) -> Self {
        let value = value.into();
        let caret = value.len();
        Self { value, caret }
    }

    /// Insert `text` at the current caret and advance the caret.
    /// Pure data mutation; the caller is responsible for
    /// invoking the change callback after this returns (the
    /// renderer's `default_render` does both in one place).
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.value.insert_str(self.caret, text);
        self.caret += text.len();
    }

    /// Delete the char immediately before the caret (Backspace).
    pub fn backspace(&mut self) {
        if self.caret == 0 {
            return;
        }
        let prev = self.prev_caret_boundary();
        self.value.drain(prev..self.caret);
        self.caret = prev;
    }

    /// Delete the char at the caret (Delete).
    pub fn delete_forward(&mut self) {
        if self.caret >= self.value.len() {
            return;
        }
        let next = self.next_caret_boundary();
        self.value.drain(self.caret..next);
    }

    /// Move the caret one char to the left, clamped to 0.
    pub fn move_caret_left(&mut self) {
        self.caret = self.prev_caret_boundary();
    }

    /// Move the caret one char to the right, clamped to len.
    pub fn move_caret_right(&mut self) {
        self.caret = self.next_caret_boundary();
    }

    /// Move the caret to the start of the value.
    pub fn move_caret_to_start(&mut self) {
        self.caret = 0;
    }

    /// Move the caret to the end of the value.
    pub fn move_caret_to_end(&mut self) {
        self.caret = self.value.len();
    }

    fn prev_caret_boundary(&self) -> usize {
        if self.caret == 0 {
            return 0;
        }
        // Walk back over the previous char's UTF-8 bytes.
        let bytes = self.value.as_bytes();
        let mut i = self.caret - 1;
        while i > 0 && (bytes[i] & 0b1100_0000) == 0b1000_0000 {
            i -= 1;
        }
        i
    }

    fn next_caret_boundary(&self) -> usize {
        let len = self.value.len();
        if self.caret >= len {
            return len;
        }
        let bytes = self.value.as_bytes();
        let mut i = self.caret + 1;
        while i < len && (bytes[i] & 0b1100_0000) == 0b1000_0000 {
            i += 1;
        }
        i
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct TextInputProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    /// `TextInputState` entity — holds the live value + caret.
    /// The renderer's key handler reads/mutates this directly.
    pub state: Entity<TextInputState>,
    /// Override placeholder text. If `state.value` is empty, the
    /// renderer paints this in the hint color instead of the
    /// real value.
    pub placeholder: String,
    pub disabled: bool,
    /// Optional hard cap on `value.len()`. Renderer enforces
    /// by silently dropping characters that would exceed it.
    pub max_length: Option<usize>,
    /// Called whenever the value changes (insert / delete /
    /// backspace). Receives the *new* value.
    pub on_change: Option<TextChangeCallback>,
    /// Called when Enter is pressed. Receives the current
    /// value.
    pub on_submit: Option<TextChangeCallback>,
    /// `true` if the caller supplied a custom background color
    /// (consumed by `TextInputRenderer.has_custom_bg`).
    pub has_custom_bg: bool,
    /// `true` if the caller supplied a custom border (consumed
    /// by `TextInputRenderer.has_custom_border`).
    pub has_custom_border: bool,
    /// `true` if the caller supplied a custom focused border
    /// (consumed by `TextInputRenderer.has_custom_focus_border`).
    pub has_custom_focus_border: bool,
    /// Caller-supplied override colors.
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn text_input(id: impl Into<ElementId>, cx: &mut App) -> TextInputProps {
    TextInputProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        state: cx.new(|_| TextInputState::new()),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        on_submit: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

pub fn text_input_with_state(
    id: impl Into<ElementId>,
    state: Entity<TextInputState>,
    cx: &mut App,
) -> TextInputProps {
    TextInputProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        state,
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        on_submit: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl TextInputProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
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

    /// Headless apply. Wires focus tracking. Key dispatch lives
    /// in the renderer (`DefaultTextInput::default_render`) so
    /// the value, caret and placeholder text are visible at
    /// paint time. The renderer needs a `&Entity<TextInputState>`
    /// which `state()` exposes.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id.clone()).track_focus(&self.focus_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_text_advances_caret() {
        let mut s = TextInputState::new();
        s.insert_text("abc");
        assert_eq!(s.value, "abc");
        assert_eq!(s.caret, 3);
    }

    #[test]
    fn backspace_removes_one_char() {
        let mut s = TextInputState::with_value("hello");
        s.backspace();
        assert_eq!(s.value, "hell");
        assert_eq!(s.caret, 4);
    }

    #[test]
    fn delete_forward_removes_one_char() {
        let mut s = TextInputState::with_value("hello");
        s.caret = 0;
        s.delete_forward();
        assert_eq!(s.value, "ello");
        assert_eq!(s.caret, 0);
    }

    #[test]
    fn caret_movement_is_char_aware() {
        let mut s = TextInputState::with_value("héllo");
        s.caret = s.value.len();
        s.move_caret_left();
        assert_eq!(&s.value[..s.caret], "héll");
        s.move_caret_left();
        assert_eq!(&s.value[..s.caret], "hél");
    }
}
