//! Tree data structures for representing hierarchical data.
//!
//! This module provides the core data types for building tree views,
//! including node types, selection modes, and tree state management.
//!
//! See the [Tree component documentation](https://github.com/MeowLynxSea/yororen-ui/wiki/Component-Tree) for usage examples.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use gpui::ElementId;

/// Newtype for tree-node identifiers. Decouples the data model
/// from `gpui::ElementId` so callers can use any `Hash + Eq`
/// type for node IDs. The conversion to / from `ElementId` lives
/// in the tree-rendering code, not in the data model.
///
/// This is the **only** type a `TreeNode` and `FlatTreeNode`'s
/// `id` field is allowed to hold — `ElementId` cannot be assigned
/// directly, which prevents accidentally passing a tab id, list
/// item id, or any other `ElementId` from elsewhere in the
/// codebase where a tree-node id is expected.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TreeNodeId(pub Cow<'static, str>);

impl TreeNodeId {
    /// Convenience for `TreeNodeId(Cow::Borrowed(s))`.
    pub fn borrowed(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
    /// Convenience for `TreeNodeId(Cow::Owned(s.into()))`.
    pub fn owned(s: impl Into<String>) -> Self {
        Self(Cow::Owned(s.into()))
    }
    /// Borrowed conversion to `gpui::ElementId`. Use this at the
    /// render boundary (e.g. `div().id(node_id.as_element_id())`)
    /// to avoid the consuming `Into<ElementId>` round-trip and
    /// keep the original `TreeNodeId` available for HashMap lookups
    /// without cloning.
    pub fn as_element_id(&self) -> ElementId {
        ElementId::Name(self.0.as_ref().to_string().into())
    }
}

impl fmt::Display for TreeNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&'static str> for TreeNodeId {
    fn from(s: &'static str) -> Self {
        Self::borrowed(s)
    }
}

impl From<String> for TreeNodeId {
    fn from(s: String) -> Self {
        Self::owned(s)
    }
}

impl From<TreeNodeId> for ElementId {
    fn from(id: TreeNodeId) -> Self {
        id.as_element_id()
    }
}

/// Selection mode for tree nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// Single selection mode - only one node can be selected at a time.
    Single,
    /// Multiple selection mode - multiple nodes can be selected.
    #[default]
    Multiple,
    /// No selection - nodes cannot be selected.
    None,
}

/// Represents a single node in a tree structure.
///
/// # Type Parameters
///
/// - `T`: The data type for this node. Defaults to [`ArcTreeNode`] for simple use cases.
///
/// # Example
///
/// ```rust,ignore
/// use yororen_ui::component::tree_data::{TreeNode, ArcTreeNode, TreeNodeBuilder};
///
/// // Using default ArcTreeNode
/// let node = TreeNodeBuilder::new("node-1", ArcTreeNode::new("My Node"))
///     .expanded(true)
///     .selected(true)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct TreeNode<T: TreeNodeData = ArcTreeNode> {
    /// Unique identifier for this node. **Strongly typed** —
    /// `TreeNodeId`, not `ElementId`, so a tree node cannot
    /// accidentally hold an id minted for some other component
    /// (tabs, list items, etc.). Convert to `ElementId` at the
    /// render boundary via [`TreeNodeId::as_element_id`].
    pub id: TreeNodeId,
    /// The data associated with this node.
    pub data: T,
    /// Child nodes of this node.
    pub children: Vec<TreeNode<T>>,
    /// Whether this node is expanded.
    pub expanded: bool,
    /// Whether this node is selected.
    pub selected: bool,
    /// Whether this node is checked (for checkbox mode).
    pub checked: TreeCheckedState,
    /// Depth level in the tree (0 for root).
    pub depth: usize,
    /// Whether this node has children.
    pub has_children: bool,
}

/// Type alias for a tree node with default [`ArcTreeNode`] data.
///
/// This is the most common use case for tree nodes.
pub type SimpleTreeNode = TreeNode<ArcTreeNode>;

/// Trait for tree node data that must be implemented by user data.
pub trait TreeNodeData: 'static + Sized + Clone {
    /// Returns the text label for this node. The return type is
    /// `Cow<'_, str>` so a node that already owns a `String` (or
    /// anything `Into<Cow<str>>`) can return it without forcing
    /// `ArcTreeNode` to clone.
    fn label(&self) -> Cow<'_, str>;
    /// Returns optional icon name for this node.
    fn icon(&self) -> Option<super::IconName> {
        None
    }
    /// Returns whether this node is disabled.
    fn disabled(&self) -> bool {
        false
    }
}

/// A simple tree node data using Arc for shared ownership.
#[derive(Debug, Clone)]
pub struct ArcTreeNode {
    pub label: String,
    pub icon: Option<String>,
    pub disabled: bool,
}

impl ArcTreeNode {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl TreeNodeData for ArcTreeNode {
    fn label(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.label)
    }

    fn icon(&self) -> Option<super::IconName> {
        // Note: IconName doesn't implement Clone, so we return None here
        // Users can implement their own TreeNodeData for custom icon support
        None
    }

    fn disabled(&self) -> bool {
        self.disabled
    }
}

/// Checked state for tree nodes with checkboxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeCheckedState {
    /// Node is unchecked.
    #[default]
    Unchecked,
    /// Node is checked.
    Checked,
    /// Node is partially checked (some children are checked).
    Indeterminate,
}

/// Builder for creating tree nodes.
pub struct TreeNodeBuilder<T: TreeNodeData = ArcTreeNode> {
    node: TreeNode<T>,
}

impl<T: TreeNodeData> TreeNodeBuilder<T> {
    /// Construct a new builder. `id` accepts anything convertible
    /// to [`TreeNodeId`] — including `&'static str` and `String`
    /// via the blanket conversions defined above. Using the
    /// newtype at the API boundary lets callers distinguish a
    /// tree-node id from an arbitrary `ElementId` from elsewhere
    /// in the codebase (tabs, list items, etc.), and prevents
    /// accidental cross-use.
    pub fn new(id: impl Into<TreeNodeId>, data: T) -> Self {
        Self {
            node: TreeNode {
                id: id.into(),
                data,
                children: Vec::new(),
                expanded: false,
                selected: false,
                checked: TreeCheckedState::Unchecked,
                depth: 0,
                has_children: false,
            },
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.node.expanded = expanded;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.node.selected = selected;
        self
    }

    pub fn checked(mut self, checked: TreeCheckedState) -> Self {
        self.node.checked = checked;
        self
    }

    pub fn child(mut self, child: TreeNode<T>) -> Self {
        self.node.has_children = true;
        self.node.children.push(child);
        self
    }

    pub fn build(self) -> TreeNode<T> {
        self.node
    }
}

/// Tree state that manages the expanded/collapsed state of nodes.
///
/// Keys are [`TreeNodeId`] (not `ElementId`) so the lookup type
/// matches the data model's node id exactly.
#[derive(Debug, Default, Clone)]
pub struct TreeState {
    pub expanded_nodes: HashMap<TreeNodeId, bool>,
    pub selected_nodes: HashMap<TreeNodeId, bool>,
    pub checked_nodes: HashMap<TreeNodeId, TreeCheckedState>,
}

impl TreeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the expanded state of a node.
    pub fn set_expanded(&mut self, id: &TreeNodeId, expanded: bool) {
        self.expanded_nodes.insert(id.clone(), expanded);
    }

    /// Check if a node is expanded.
    pub fn is_expanded(&self, id: &TreeNodeId) -> bool {
        self.expanded_nodes.get(id).copied().unwrap_or(false)
    }

    /// Set the selected state of a node.
    pub fn set_selected(&mut self, id: &TreeNodeId, selected: bool) {
        self.selected_nodes.insert(id.clone(), selected);
    }

    /// Check if a node is selected.
    pub fn is_selected(&self, id: &TreeNodeId) -> bool {
        self.selected_nodes.get(id).copied().unwrap_or(false)
    }

    /// Set the checked state of a node.
    pub fn set_checked(&mut self, id: &TreeNodeId, checked: TreeCheckedState) {
        self.checked_nodes.insert(id.clone(), checked);
    }

    /// Get the checked state of a node.
    pub fn get_checked(&self, id: &TreeNodeId) -> TreeCheckedState {
        self.checked_nodes
            .get(id)
            .copied()
            .unwrap_or(TreeCheckedState::Unchecked)
    }

    /// Toggle the expanded state of a node.
    pub fn toggle_expanded(&mut self, id: &TreeNodeId) {
        let current = self.is_expanded(id);
        self.set_expanded(id, !current);
    }

    /// Clear all selected nodes.
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    /// Get all selected node IDs.
    pub fn selected_ids(&self) -> impl Iterator<Item = &TreeNodeId> {
        self.selected_nodes.keys()
    }

    /// Get all checked node IDs.
    pub fn checked_ids(&self) -> impl Iterator<Item = (&TreeNodeId, &TreeCheckedState)> {
        self.checked_nodes.iter()
    }

    /// Get all expanded node IDs.
    pub fn expanded_nodes(&self) -> impl Iterator<Item = (&TreeNodeId, &bool)> {
        self.expanded_nodes.iter()
    }
}

/// Trait for customizing tree node rendering.
pub trait TreeNodeRenderer<T: TreeNodeData>: 'static + Sized {
    /// Render the content of a tree node.
    fn render_node(
        &self,
        node: &TreeNode<T>,
        expanded: bool,
        selected: bool,
        checked: TreeCheckedState,
    ) -> gpui::AnyElement;
}

// --- Tree-specific callback types ----------------------------------------
//
// The generic `ElementClickCallback` / `ElementMouseDownCallback` /
// `ElementCallback` aliases (in `crate::component::callback`) carry an
// `&ElementId` because the rest of the UI needs an id that can be turned
// into a `gpui::ElementId` for state-keyed storage. Trees carry
// `TreeNodeId` instead, so they need their own callback shapes that
// surface the strongly-typed id to the handler.

/// Click callback for a tree row. Carries the clicked node's
/// [`TreeNodeId`] (not an `ElementId`).
pub type TreeItemClickCallback<T = gpui::ClickEvent> =
    std::sync::Arc<dyn Fn(&TreeNodeId, &T, &mut gpui::Window, &mut gpui::App)>;

/// Right-click / context-menu callback for a tree row.
pub type TreeItemContextMenuCallback =
    std::sync::Arc<dyn Fn(&TreeNodeId, &gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App)>;

/// "Node id only" callback for tree events (toggle_expand, select).
pub type TreeIdCallback = std::sync::Arc<dyn Fn(&TreeNodeId)>;

/// Checkbox change callback (node id + new checked state).
pub type TreeCheckCallback =
    std::sync::Arc<dyn Fn(&TreeNodeId, TreeCheckedState)>;

/// Flattened tree node for virtualized rendering.
///
/// This is used internally by the Tree component to render only visible nodes.
#[derive(Debug, Clone)]
pub struct FlatTreeNode<T: TreeNodeData = ArcTreeNode> {
    /// The original node ID. Strongly typed — see [`TreeNode::id`].
    pub id: TreeNodeId,
    /// The node data.
    pub data: T,
    /// Depth level in the tree.
    pub depth: usize,
    /// Whether this node is expanded.
    pub expanded: bool,
    /// Whether this node is selected.
    pub selected: bool,
    /// Whether this node is checked.
    pub checked: TreeCheckedState,
    /// Whether this node has children.
    pub has_children: bool,
    /// Index in the flattened list.
    pub index: usize,
}

/// Type alias for a flattened tree node with default [`ArcTreeNode`] data.
pub type SimpleFlatTreeNode = FlatTreeNode<ArcTreeNode>;

/// Flattens a tree structure into a list for virtualized rendering.
pub fn flatten_tree<T: TreeNodeData>(
    nodes: &[TreeNode<T>],
    expanded_ids: &HashMap<TreeNodeId, bool>,
    include_hidden: bool,
) -> Vec<FlatTreeNode<T>> {
    let mut result = Vec::new();
    flatten_tree_recursive(nodes, expanded_ids, 0, &mut result, include_hidden, 0);
    result
}

fn flatten_tree_recursive<T: TreeNodeData>(
    nodes: &[TreeNode<T>],
    expanded_ids: &HashMap<TreeNodeId, bool>,
    depth: usize,
    result: &mut Vec<FlatTreeNode<T>>,
    include_hidden: bool,
    start_index: usize,
) -> usize {
    let mut index = start_index;

    for node in nodes {
        let is_expanded = expanded_ids.get(&node.id).copied().unwrap_or(node.expanded);

        // If not expanded and not including hidden nodes, skip this node's children
        if !is_expanded && !include_hidden {
            // Still add the node but mark as not expanded
            result.push(FlatTreeNode {
                id: node.id.clone(),
                data: node.data.clone(),
                depth,
                expanded: is_expanded,
                selected: node.selected,
                checked: node.checked,
                has_children: node.has_children,
                index,
            });
            index += 1;
            continue;
        }

        result.push(FlatTreeNode {
            id: node.id.clone(),
            data: node.data.clone(),
            depth,
            expanded: is_expanded,
            selected: node.selected,
            checked: node.checked,
            has_children: node.has_children,
            index,
        });
        index += 1;

        // Recursively flatten children if expanded
        if is_expanded && !node.children.is_empty() {
            index = flatten_tree_recursive(
                &node.children,
                expanded_ids,
                depth + 1,
                result,
                include_hidden,
                index,
            );
        }
    }

    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_node_builder_accepts_static_str() {
        let node = TreeNodeBuilder::new("root", ArcTreeNode::new("Root")).build();
        assert_eq!(node.id, TreeNodeId::borrowed("root"));
    }

    #[test]
    fn tree_node_builder_accepts_owned_string() {
        let id = String::from("dynamic-1");
        let node = TreeNodeBuilder::new(id.clone(), ArcTreeNode::new("Dynamic")).build();
        assert_eq!(node.id, TreeNodeId::owned(id));
    }

    #[test]
    fn tree_node_builder_accepts_typed_tree_node_id() {
        let id = TreeNodeId::owned("typed-id");
        let node = TreeNodeBuilder::new(id.clone(), ArcTreeNode::new("Typed")).build();
        assert_eq!(node.id, id);
    }

    #[test]
    fn tree_node_id_field_rejects_element_id_assignment() {
        // The id is now `TreeNodeId`, not `ElementId` — this is a
        // **compile-time** guarantee. Verify the field type is
        // `TreeNodeId` by constructing the struct literally.
        let node: TreeNode = TreeNode {
            id: TreeNodeId::borrowed("static-check"),
            data: ArcTreeNode::new("data"),
            children: Vec::new(),
            expanded: false,
            selected: false,
            checked: TreeCheckedState::Unchecked,
            depth: 0,
            has_children: false,
        };
        assert_eq!(node.id.to_string(), "static-check");
    }

    #[test]
    fn as_element_id_borrowed_does_not_consume() {
        let id = TreeNodeId::borrowed("non-consuming");
        // Call as_element_id twice — the original is still usable.
        let _ = id.as_element_id();
        let _ = id.as_element_id();
        assert_eq!(id.to_string(), "non-consuming");
    }

    #[test]
    fn display_via_cow() {
        // Both borrowed and owned variants implement Display so
        // tuple-style ElementId construction (`(id, suffix).into()`)
        // works at the render boundary.
        let borrowed = TreeNodeId::borrowed("a");
        let owned = TreeNodeId::owned("b".to_string());
        assert_eq!(format!("{borrowed}"), "a");
        assert_eq!(format!("{owned}"), "b");
    }
}
