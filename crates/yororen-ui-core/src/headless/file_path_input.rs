//! Headless `file_path_input` — text input specialised for
//! filesystem paths, with a folder-icon and a "browse" button.

use std::sync::Arc;

use gpui::{App, Hsla};

#[derive(Clone)]
pub struct FilePathInputProps {
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub value: String,
    pub on_change: Option<Arc<dyn Fn(&str, &mut gpui::Window, &mut App) + Send + Sync>>,
    pub on_browse: Option<Arc<dyn Fn(&mut gpui::Window, &mut App) + Send + Sync>>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn file_path_input(id: impl Into<gpui::ElementId>) -> FilePathInputProps {
    FilePathInputProps {
        id: id.into(),
        placeholder: "/path/to/file".to_string(),
        disabled: false,
        value: String::new(),
        on_change: None,
        on_browse: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl FilePathInputProps {
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
        F: 'static + Send + Sync + Fn(&str, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_browse<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut gpui::Window, &mut App),
    {
        self.on_browse = Some(Arc::new(f));
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
