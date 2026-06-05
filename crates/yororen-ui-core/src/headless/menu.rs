//! Headless `menu` — like `dropdown_menu` but rendered without a
//! trigger (used inside `popover`s or as a context menu body).

use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful};

use super::dropdown_menu::{DropdownItem, DropdownMenuItem, DropdownSelectCallback};

#[derive(Clone)]
pub struct MenuState {
    pub highlighted_index: Option<usize>,
    pub items: Vec<DropdownItem>,
    on_select: Option<DropdownSelectCallback>,
}

impl MenuState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            highlighted_index: None,
            items: Vec::new(),
            on_select: None,
        })
    }

    pub fn set_items(&mut self, items: Vec<DropdownItem>) {
        self.items = items;
    }
    pub fn highlight_next(&mut self) {
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
            && let Some(DropdownItem::Item(DropdownMenuItem { id, .. })) = self.items.get(i)
        {
            let id = id.clone();
            if let Some(f) = &self.on_select {
                f(id, window, cx);
            }
        }
    }
}

#[derive(Clone)]
pub struct MenuProps {
    pub id: ElementId,
    pub state: Entity<MenuState>,
}

pub fn menu(id: impl Into<ElementId>, state: Entity<MenuState>) -> MenuProps {
    MenuProps {
        id: id.into(),
        state,
    }
}

impl MenuProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
