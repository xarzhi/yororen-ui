//! Headless `search_input` — text input with a search-icon
//! and a clear-button. Reuses `TextInputState`.

use std::sync::Arc;

use gpui::{
    App, AppContext, Div, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use super::text_input::TextInputState;

pub type SearchChangeCallback =
    Arc<dyn Fn(&str, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct SearchInputProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub state: Entity<TextInputState>,
    pub placeholder: String,
    pub disabled: bool,
    pub value: String,
    pub on_change: Option<SearchChangeCallback>,
    pub on_submit: Option<SearchChangeCallback>,
    pub on_clear: Option<Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn search_input(id: impl Into<ElementId>, cx: &mut App) -> SearchInputProps {
    SearchInputProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        state: cx.new(|_| TextInputState::new()),
        placeholder: "Search…".to_string(),
        disabled: false,
        value: String::new(),
        on_change: None,
        on_submit: None,
        on_clear: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl SearchInputProps {
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
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
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
    pub fn on_clear<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut Window, &mut App),
    {
        self.on_clear = Some(Arc::new(f));
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
