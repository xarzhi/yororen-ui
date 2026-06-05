//! Headless `select` — option list with a current value and
//! keyboard navigation. The renderer draws a trigger + dropdown.

use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct SelectOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}

pub type SelectChangeCallback = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct SelectState {
    pub open: bool,
    pub value: Option<SharedString>,
    pub options: Vec<SelectOption>,
    pub highlighted_index: Option<usize>,
    pub placeholder: SharedString,
    pub dismiss_on_escape: bool,
    on_change: Option<SelectChangeCallback>,
}

impl SelectState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            open: false,
            value: None,
            options: Vec::new(),
            highlighted_index: None,
            placeholder: "Select an option…".into(),
            dismiss_on_escape: true,
            on_change: None,
        })
    }

    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn set_options(&mut self, opts: Vec<SelectOption>) {
        self.options = opts;
    }
    pub fn set_value(&mut self, v: impl Into<SharedString>) {
        self.value = Some(v.into());
    }
    pub fn set_placeholder(&mut self, p: impl Into<SharedString>) {
        self.placeholder = p.into();
    }
    pub fn highlight(&mut self, i: usize) {
        if i < self.options.len() {
            self.highlighted_index = Some(i);
        }
    }
    pub fn highlight_next(&mut self) {
        let next = match self.highlighted_index {
            Some(i) if i + 1 < self.options.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.highlighted_index = Some(next);
    }
    pub fn highlight_prev(&mut self) {
        let prev = match self.highlighted_index {
            Some(0) | None => self.options.len().saturating_sub(1),
            Some(i) => i - 1,
        };
        self.highlighted_index = Some(prev);
    }
    pub fn set_on_change<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
    }
    pub fn invoke_change(&self, value: SharedString, window: &mut gpui::Window, cx: &mut App) {
        if let Some(f) = &self.on_change {
            f(value, window, cx);
        }
    }
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(opt) = self.options.get(i)
        {
            let value = opt.value.clone();
            self.value = Some(value.clone());
            self.open = false;
            self.invoke_change(value, window, cx);
        }
    }
}

#[derive(Clone)]
pub struct SelectProps {
    pub id: ElementId,
    pub state: Entity<SelectState>,
}

pub fn select(id: impl Into<ElementId>, state: Entity<SelectState>) -> SelectProps {
    SelectProps {
        id: id.into(),
        state,
    }
}

impl SelectProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
