//! Headless `listbox` — a scrollable single-select list with
//! keyboard navigation. Shared algorithm with `select` /
//! `combo_box` / `menu` lives in
//! [`crate::headless::list_navigable`]; this module just owns
//! the listbox-specific state, options, and the data-layer
//! `select_highlighted` action.

use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful};

use super::list_navigable::{ListNavigable, highlight_next, highlight_prev};

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
        highlight_next(self);
    }
    pub fn highlight_prev(&mut self) {
        highlight_prev(self);
    }
    pub fn set_on_change<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
    }
    /// Pick the currently highlighted option. Headless data-layer
    /// action — does not belong in the renderer. Writes the
    /// selected value and fires the user-supplied `on_change`
    /// callback. The renderer composes visuals + wires this as
    /// each row's click / Enter handler.
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(opt) = self.options.get(i)
            && !opt.disabled
        {
            let v = opt.value.clone();
            self.selected_value = Some(v.clone());
            if let Some(f) = &self.on_change {
                f(v, window, cx);
            }
        }
    }
    /// Pick a specific option by value. Headless data-layer
    /// action — does not belong in the renderer. Identical
    /// side-effect to `select_highlighted` but takes the value
    /// directly, so a click handler that already knows which
    /// row was hit can fire `on_change` without having to first
    /// mutate `highlighted_index`.
    pub fn pick(&mut self, value: SharedString, window: &mut gpui::Window, cx: &mut App) {
        if self.options.iter().any(|o| o.value == value) {
            self.selected_value = Some(value.clone());
            if let Some(f) = &self.on_change {
                f(value, window, cx);
            }
        }
    }
}

impl ListNavigable for ListboxState {
    fn options_len(&self) -> usize {
        self.options.len()
    }
    fn highlighted_index(&self) -> Option<usize> {
        self.highlighted_index
    }
    fn set_highlighted(&mut self, i: usize) {
        self.highlighted_index = Some(i);
    }
    /// Disabled options are visible but not selectable. The
    /// shared `highlight_next` / `highlight_prev` will skip over
    /// them rather than land on them.
    fn is_selectable(&self, i: usize) -> bool {
        self.options
            .get(i)
            .map(|o| !o.disabled)
            .unwrap_or(false)
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
    /// Apply the headless contract to the renderer-built `el`.
    /// Sets the element id only. The renderer is responsible
    /// for visuals AND for wiring each row's click handler to
    /// `state.pick(value, …)` (or to mutate `highlighted_index`
    /// and then call `state.select_highlighted(…)`).
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the listbox via the registered `ListboxRenderer`.
    /// Returns a `Stateful<Div>` containing one row per option,
    /// with the highlighted row styled accordingly and the
    /// selected row marked. Clicking a row fires
    /// `state.pick(value, …)`; the registered renderer decides
    /// visual treatment (backgrounds, hover, dividers, etc.).
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::listbox::ListboxRenderer;
        use crate::renderer::markers::Listbox as ListboxMarker;

        let r: &Arc<dyn ListboxRenderer> = cx
            .renderer_arc::<ListboxMarker, dyn ListboxRenderer>()
            .expect("ListboxRenderer registered");
        // `ListboxRenderer::compose` returns the full
        // `Stateful<Div>` (it iterates the options itself, so
        // the shell is complete). `apply` is for callers who
        // built a `Div` themselves and want the headless `id`
        // stamped on top; the renderer-built path returns
        // `compose`'s result directly.
        r.compose(&self, cx)
    }
}