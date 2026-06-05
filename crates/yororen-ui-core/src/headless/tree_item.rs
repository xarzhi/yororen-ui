//! Headless `tree_item` — a single row in a `tree`. Pure data.

use std::sync::Arc;

use gpui::{App, ClickEvent, Div, ElementId, FocusHandle, SharedString, Stateful};

pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut gpui::Window, &mut App) + Send + Sync>;

#[derive(Clone, Debug)]
pub struct TreeNodeId(pub SharedString);

impl TreeNodeId {
    pub fn new(s: impl Into<SharedString>) -> Self {
        Self(s.into())
    }
}

impl From<&'static str> for TreeNodeId {
    fn from(s: &'static str) -> Self {
        Self(s.into())
    }
}

impl From<String> for TreeNodeId {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

#[derive(Clone)]
pub struct TreeItemProps {
    pub id: ElementId,
    pub node_id: TreeNodeId,
    pub label: String,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
    pub selected: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_click: Option<ClickCallback>,
    pub on_toggle: Option<ClickCallback>,
}

pub fn tree_item(
    id: impl Into<ElementId>,
    node_id: impl Into<TreeNodeId>,
    label: impl Into<String>,
    cx: &mut App,
) -> TreeItemProps {
    TreeItemProps {
        id: id.into(),
        node_id: node_id.into(),
        label: label.into(),
        depth: 0,
        has_children: false,
        expanded: false,
        selected: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_click: None,
        on_toggle: None,
    }
}

impl TreeItemProps {
    pub fn depth(mut self, d: usize) -> Self {
        self.depth = d;
        self
    }
    pub fn has_children(mut self, v: bool) -> Self {
        self.has_children = v;
        self
    }
    pub fn expanded(mut self, v: bool) -> Self {
        self.expanded = v;
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut App),
    {
        self.on_click = Some(Arc::new(f));
        self
    }
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(f));
        self
    }
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if !self.disabled
            && let Some(f) = self.on_click.clone()
        {
            s = s.on_click(move |ev, window, cx| f(ev, window, cx));
        }
        s
    }
}
