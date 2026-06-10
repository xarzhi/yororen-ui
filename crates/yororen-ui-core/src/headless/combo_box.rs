//! Headless `combo_box` — text input + option list + keyboard
//! navigation. The renderer shows a text field + a dropdown list.

use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct ComboBoxOption {
    pub value: SharedString,
    pub label: SharedString,
}

impl ComboBoxOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

pub type ComboBoxChangeCallback = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct ComboBoxState {
    pub open: bool,
    pub text: String,
    pub value: Option<SharedString>,
    pub options: Vec<ComboBoxOption>,
    pub highlighted_index: Option<usize>,
    pub placeholder: SharedString,
    pub dismiss_on_escape: bool,
    on_change: Option<ComboBoxChangeCallback>,
}

impl ComboBoxState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            open: false,
            text: String::new(),
            value: None,
            options: Vec::new(),
            highlighted_index: None,
            placeholder: "Search…".into(),
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
    pub fn set_text(&mut self, t: impl Into<String>) {
        self.text = t.into();
    }
    pub fn set_options(&mut self, opts: Vec<ComboBoxOption>) {
        self.options = opts;
    }
    pub fn set_value(&mut self, v: impl Into<SharedString>) {
        let v = v.into();
        self.value = Some(v.clone());
        if let Some(opt) = self.options.iter().find(|o| o.value == v) {
            self.text = opt.label.to_string();
        }
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
            self.text = opt.label.to_string();
            self.value = Some(value.clone());
            self.open = false;
            self.invoke_change(value, window, cx);
        }
    }
}

#[derive(Clone)]
pub struct ComboBoxProps {
    pub id: ElementId,
    pub state: Entity<ComboBoxState>,
}

pub fn combo_box(id: impl Into<ElementId>, state: Entity<ComboBoxState>) -> ComboBoxProps {
    ComboBoxProps {
        id: id.into(),
        state,
    }
}

impl ComboBoxProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the combo box using the registered
    /// `ComboBoxRenderer`. Returns a `Stateful<Div>` with the
    /// element id. The renderer decides bg / border / padding
    /// based on the `state` entity.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::combo_box::ComboBoxRenderer;
        use crate::renderer::markers::ComboBox as ComboBoxMarker;

        let r: &Arc<dyn ComboBoxRenderer> = cx
            .renderer_arc::<ComboBoxMarker, dyn ComboBoxRenderer>()
            .expect("ComboBoxRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
