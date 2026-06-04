//! \`DropdownMenuRoot\` — composite API for dropdown menus.

use gpui::{ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, div, ParentElement};

use crate::component::dropdown_menu::{DropdownItem, DropdownMenu, DropdownMenuItem};
use crate::component::popover::PopoverPlacement;

/// \`DropdownMenuRoot\` is the split-API form of \`dropdown_menu()\`.
pub struct DropdownMenuRoot {
    id: ElementId,
    label: SharedString,
    items: Vec<DropdownItem>,
    open: bool,
    width: Option<gpui::Pixels>,
    placement: PopoverPlacement,
    on_select: Option<Box<dyn Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App)>>,
}

impl DropdownMenuRoot {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: "Menu".into(),
            items: Vec::new(),
            open: false,
            width: None,
            placement: PopoverPlacement::BottomStart,
            on_select: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Add a single menu item by id + label.
    pub fn item(mut self, id: impl Into<String>, label: impl Into<SharedString>) -> Self {
        self.items.push(DropdownItem::Item(
            DropdownMenuItem::new(id, label),
        ));
        self
    }

    /// Add a pre-built \`DropdownItem\` (item or separator).
    pub fn push(mut self, item: DropdownItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set the menu items at once. Clears any previously added
    /// items.
    pub fn items(mut self, items: impl IntoIterator<Item = DropdownItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_select = Some(Box::new(f));
        self
    }

    /// \`trigger\` / \`content\` are accepted for API symmetry
    /// with the other Root types but dropdown menus are
    /// self-contained (the label is the trigger), so these
    /// methods are no-ops.
    pub fn trigger(self, _trigger: impl IntoElement) -> Self {
        self
    }
    pub fn content(self, _content: impl IntoElement) -> Self {
        self
    }
}

impl RenderOnce for DropdownMenuRoot {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let mut m = DropdownMenu::new(self.id)
            .label(self.label)
            .items(self.items)
            .open(self.open)
            .placement(self.placement);
        if let Some(w) = self.width {
            m = m.width(w);
        }
        if let Some(f) = self.on_select {
            m = m.on_select(f);
        }
        div().child(m)
    }
}

/// \`DropdownMenuTrigger\` — convenience alias.
pub struct DropdownMenuTrigger {
    id: ElementId,
}

impl DropdownMenuTrigger {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
    pub fn id(&self) -> ElementId {
        self.id.clone()
    }
}

/// \`DropdownMenuContent\` — convenience alias.
pub struct DropdownMenuContent {
    id: ElementId,
}

impl DropdownMenuContent {
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
    fn dropdown_menu_root_defaults() {
        let r = DropdownMenuRoot::new("d1");
        assert!(!r.open);
        assert_eq!(r.label.to_string(), "Menu");
    }

    #[test]
    fn dropdown_menu_root_items() {
        let r = DropdownMenuRoot::new("d1")
            .item("save", "Save")
            .item("open", "Open");
        assert_eq!(r.items.len(), 2);
    }
}
