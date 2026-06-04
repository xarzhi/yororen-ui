//! \`SelectRoot\` — composite API for selects.

use gpui::{ElementId, IntoElement, RenderOnce, SharedString, div, ParentElement};

use crate::component::select::{Select, SelectOption};

/// \`SelectRoot\` is the split-API form of \`select()\`.
pub struct SelectRoot {
    id: ElementId,
    value: String,
    options: Vec<SelectOption>,
    placeholder: Option<SharedString>,
    disabled: bool,
    on_change: Option<Box<dyn Fn(String, &mut gpui::Window, &mut gpui::App)>>,
}

impl SelectRoot {
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
        self.options.push(SelectOption::new().value(value).label(label));
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption>) -> Self {
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
        F: 'static + Fn(String, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Box::new(f));
        self
    }

    /// \`trigger\` / \`content\` are accepted for API symmetry
    /// with the other Root types but \`select\` is a single
    /// element. They are no-ops.
    pub fn trigger(self, _trigger: impl IntoElement) -> Self {
        self
    }
    pub fn content(self, _content: impl IntoElement) -> Self {
        self
    }
}

impl RenderOnce for SelectRoot {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let mut s = Select::new().id(self.id).value(self.value).options(self.options);
        if let Some(p) = self.placeholder {
            s = s.placeholder(p);
        }
        if self.disabled {
            s = s.disabled(true);
        }
        if let Some(f) = self.on_change {
            s = s.on_change(f);
        }
        div().child(s)
    }
}

/// \`SelectTrigger\` — convenience alias.
pub struct SelectTrigger {
    id: ElementId,
}

impl SelectTrigger {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
    pub fn id(&self) -> ElementId {
        self.id.clone()
    }
}

/// \`SelectContent\` — convenience alias.
pub struct SelectContent {
    id: ElementId,
}

impl SelectContent {
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
    fn select_root_defaults() {
        let r = SelectRoot::new("s1");
        assert_eq!(r.value, "");
        assert!(r.options.is_empty());
    }

    #[test]
    fn select_root_options() {
        let r = SelectRoot::new("s1")
            .option("a", "Apple")
            .option("b", "Banana");
        assert_eq!(r.options.len(), 2);
    }
}
