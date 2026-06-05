//! Tree Expanded Demo State
//!
//! Holds the TreeState (expansion + selection) and a generation counter
//! so we can simulate "new nodes arriving" while preserving user choices.

use gpui::{App, AppContext, Entity, Global};

use yororen_ui::headless::{ArcTreeNode, TreeCheckedState, TreeNode, TreeNodeId};

/// Global state for the tree demo.
pub struct TreeDemoState {
    /// Generation counter incremented every time we "replace" nodes.
    pub generation: Entity<u32>,
}

impl TreeDemoState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            generation: cx.new(|_| 0),
        }
    }
}

impl Global for TreeDemoState {}

fn make_node(id: impl Into<TreeNodeId>, label: impl Into<String>, expanded: bool) -> TreeNode {
    TreeNode {
        id: id.into(),
        data: ArcTreeNode::new(label),
        children: Vec::new(),
        expanded,
        selected: false,
        checked: TreeCheckedState::Unchecked,
        depth: 0,
        has_children: false,
    }
}

/// Build a fresh set of nodes where some are initially expanded.
/// The `generation` number is baked into labels so you can visually
/// confirm that nodes were recreated.
pub fn build_nodes(generation: u32) -> Vec<TreeNode> {
    let leaf_a1 = make_node("leaf-a1", format!("Leaf A1 (gen {})", generation), false);
    let leaf_a2 = make_node("leaf-a2", format!("Leaf A2 (gen {})", generation), false);
    let leaf_b1 = make_node("leaf-b1", format!("Leaf B1 (gen {})", generation), false);

    let mut child_a = make_node("child-a", format!("Child A (gen {})", generation), true);
    child_a.children.push(leaf_a1);
    child_a.children.push(leaf_a2);
    child_a.has_children = true;

    let mut child_b = make_node("child-b", format!("Child B (gen {})", generation), true);
    child_b.children.push(leaf_b1);
    child_b.has_children = true;

    let mut root = make_node("root", format!("Root (gen {})", generation), true);
    root.children.push(child_a);
    root.children.push(child_b);
    root.has_children = true;

    vec![root]
}
