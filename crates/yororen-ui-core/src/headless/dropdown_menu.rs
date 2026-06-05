//! Headless `dropdown_menu` — a vertical list of items triggered
//! by a button.

use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub enum DropdownItem {
    Item(DropdownMenuItem),
    Separator,
    Group(DropdownMenuGroup),
}

#[derive(Clone, Debug)]
pub struct DropdownMenuItem {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub disabled: bool,
    pub shortcut: Option<Vec<String>>,
}

impl DropdownMenuItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            shortcut: None,
        }
    }
    pub fn icon(mut self, i: impl Into<SharedString>) -> Self {
        self.icon = Some(i.into());
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn shortcut(mut self, k: Vec<String>) -> Self {
        self.shortcut = Some(k);
        self
    }
}

#[derive(Clone, Debug)]
pub struct DropdownMenuGroup {
    pub label: SharedString,
    pub items: Vec<DropdownMenuItem>,
}

pub type DropdownSelectCallback = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct DropdownMenuState {
    pub open: bool,
    pub highlighted_index: Option<usize>,
    pub dismiss_on_escape: bool,
    pub items: Vec<DropdownItem>,
    on_select: Option<DropdownSelectCallback>,
}

impl DropdownMenuState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            open: false,
            highlighted_index: None,
            dismiss_on_escape: true,
            items: Vec::new(),
            on_select: None,
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
    pub fn set_items(&mut self, items: Vec<DropdownItem>) {
        self.items = items;
    }
    pub fn highlight_next(&mut self) {
        // Skip separators.
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let mut i = match self.highlighted_index {
            Some(i) => i + 1,
            None => 0,
        };
        while i < len && matches!(self.items[i], DropdownItem::Separator) {
            i += 1;
        }
        if i < len {
            self.highlighted_index = Some(i);
        }
    }
    pub fn highlight_prev(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let mut i = match self.highlighted_index {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        while i > 0 && matches!(self.items[i], DropdownItem::Separator) {
            i -= 1;
        }
        self.highlighted_index = Some(i);
    }
    pub fn set_on_select<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_select = Some(Arc::new(f));
    }
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(DropdownItem::Item(it)) = self.items.get(i)
        {
            let id = it.id.clone();
            self.open = false;
            if let Some(f) = &self.on_select {
                f(id, window, cx);
            }
        }
    }
}

#[derive(Clone)]
pub struct DropdownMenuProps {
    pub id: ElementId,
    pub state: Entity<DropdownMenuState>,
}

pub fn dropdown_menu(
    id: impl Into<ElementId>,
    state: Entity<DropdownMenuState>,
) -> DropdownMenuProps {
    DropdownMenuProps {
        id: id.into(),
        state,
    }
}

impl DropdownMenuProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
