//! Headless `tree` — the data structure of a tree (parent/child
//! relationships) and a current selection + expansion set. The
//! visual lives in the renderer; callers iterate `children(id)` to
//! produce rows.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use gpui::{App, Div, ElementId, SharedString, Stateful};

use super::tree_item::TreeNodeId;

#[derive(Clone, Debug, Default)]
pub struct TreeData {
    pub children: BTreeMap<TreeNodeId, Vec<TreeNodeId>>,
    pub labels: HashMap<TreeNodeId, String>,
    pub disabled: BTreeSet<TreeNodeId>,
    pub roots: Vec<TreeNodeId>,
}

impl TreeData {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, parent: Option<TreeNodeId>, id: TreeNodeId, label: impl Into<String>) {
        if let Some(p) = parent {
            self.children.entry(p).or_default().push(id.clone());
        } else {
            self.roots.push(id.clone());
        }
        self.labels.insert(id, label.into());
    }
    pub fn children_of(&self, id: &TreeNodeId) -> &[TreeNodeId] {
        self.children.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }
    pub fn label_of(&self, id: &TreeNodeId) -> Option<&str> {
        self.labels.get(id).map(String::as_str)
    }
    pub fn is_disabled(&self, id: &TreeNodeId) -> bool {
        self.disabled.contains(id)
    }
}

#[derive(Clone)]
pub struct TreeProps {
    pub id: ElementId,
    pub data: TreeData,
    pub expanded: BTreeSet<TreeNodeId>,
    pub selected: Option<TreeNodeId>,
}

pub fn tree(id: impl Into<ElementId>, _cx: &mut App) -> TreeProps {
    TreeProps {
        id: id.into(),
        data: TreeData::new(),
        expanded: BTreeSet::new(),
        selected: None,
    }
}

impl TreeProps {
    pub fn data(mut self, d: TreeData) -> Self {
        self.data = d;
        self
    }
    pub fn expanded(mut self, id: impl Into<TreeNodeId>) -> Self {
        self.expanded.insert(id.into());
        self
    }
    pub fn selected(mut self, id: impl Into<TreeNodeId>) -> Self {
        self.selected = Some(id.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}

/// Re-export the helper type used by `tree_item` so callers don't
/// have to know the headless module layout.
pub use super::tree_item::TreeNodeId as _TreeNodeId;
/// Convenience: a node id from a static `&str` or a `SharedString`.
pub fn node_id(s: impl Into<SharedString>) -> TreeNodeId {
    TreeNodeId(s.into())
}
