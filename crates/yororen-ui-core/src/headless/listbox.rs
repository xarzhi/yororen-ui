//! Headless `listbox` — a list of options with keyboard nav and
//! optional multi-select. Used by `select` / `menu` and exposed
//! directly for custom surfaces.

use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct ListboxOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl ListboxOption {
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

pub type ListboxChangeCallback = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct ListboxState {
    pub options: Vec<ListboxOption>,
    pub highlighted_index: Option<usize>,
    pub selected_value: Option<SharedString>,
    on_change: Option<ListboxChangeCallback>,
}

impl ListboxState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            options: Vec::new(),
            highlighted_index: None,
            selected_value: None,
            on_change: None,
        })
    }

    pub fn set_options(&mut self, opts: Vec<ListboxOption>) {
        self.options = opts;
    }
    pub fn set_selected(&mut self, v: impl Into<SharedString>) {
        self.selected_value = Some(v.into());
    }
    pub fn highlight_next(&mut self) {
        let n = self.options.len();
        if n == 0 {
            return;
        }
        self.highlighted_index = Some(match self.highlighted_index {
            Some(i) if i + 1 < n => i + 1,
            Some(_) => 0,
            None => 0,
        });
    }
    pub fn highlight_prev(&mut self) {
        let n = self.options.len();
        if n == 0 {
            return;
        }
        self.highlighted_index = Some(match self.highlighted_index {
            Some(0) | None => n - 1,
            Some(i) => i - 1,
        });
    }
    pub fn set_on_change<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
    }
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(opt) = self.options.get(i)
        {
            let v = opt.value.clone();
            self.selected_value = Some(v.clone());
            if let Some(f) = &self.on_change {
                f(v, window, cx);
            }
        }
    }
}

#[derive(Clone)]
pub struct ListboxProps {
    pub id: ElementId,
    pub state: Entity<ListboxState>,
}

pub fn listbox(id: impl Into<ElementId>, state: Entity<ListboxState>) -> ListboxProps {
    ListboxProps {
        id: id.into(),
        state,
    }
}

impl ListboxProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
