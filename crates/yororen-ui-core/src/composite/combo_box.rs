//! \`ComboBoxRoot\` — composite API for combo boxes.

use gpui::{ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, div, ParentElement};

use crate::component::combo_box::{ComboBox, ComboBoxOption};

/// \`ComboBoxRoot\` is the split-API form of \`combo_box()\`.
pub struct ComboBoxRoot {
    id: ElementId,
    value: String,
    options: Vec<ComboBoxOption>,
    placeholder: Option<SharedString>,
    disabled: bool,
    on_change: Option<Box<dyn Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App)>>,
}

impl ComboBoxRoot {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: String::new(),
            options: Vec::new(),
            placeholder: None,
            disabled: false,
            on_change: None,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn option(
        mut self,
        value: impl Into<String>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.options
            .push(ComboBoxOption::new(value, label));
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = ComboBoxOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn trigger(self, _trigger: impl IntoElement) -> Self {
        self
    }
    pub fn content(self, _content: impl IntoElement) -> Self {
        self
    }
}

impl RenderOnce for ComboBoxRoot {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let mut c = ComboBox::new()
            .id(self.id)
            .value(self.value)
            .options(self.options);
        if let Some(p) = self.placeholder {
            c = c.placeholder(p);
        }
        if self.disabled {
            c = c.disabled(true);
        }
        if let Some(f) = self.on_change {
            c = c.on_change(f);
        }
        div().child(c)
    }
}

/// \`ComboBoxTrigger\` — convenience alias.
pub struct ComboBoxTrigger {
    id: ElementId,
}

impl ComboBoxTrigger {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
    pub fn id(&self) -> ElementId {
        self.id.clone()
    }
}

/// \`ComboBoxContent\` — convenience alias.
pub struct ComboBoxContent {
    id: ElementId,
}

impl ComboBoxContent {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
    pub fn id(&self) -> ElementId {
        self.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combo_box_root_defaults() {
        let r = ComboBoxRoot::new("c1");
        assert_eq!(r.value, "");
        assert!(r.options.is_empty());
    }
}
