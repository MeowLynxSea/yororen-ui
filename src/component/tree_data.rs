//! Tree data structures for representing hierarchical data.
//!
//! This module provides the core data types for building tree views,
//! including node types, selection modes, and tree state management.
//!
//! See the [Tree component documentation](https://github.com/MeowLynxSea/yororen-ui/wiki/Component-Tree) for usage examples.

use gpui::ElementId;
use std::collections::HashMap;

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
    /// Unique identifier for this node.
    pub id: ElementId,
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
    /// Returns the text label for this node.
    fn label(&self) -> &str;
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
    fn label(&self) -> &str {
        &self.label
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
    pub fn new(id: impl Into<ElementId>, data: T) -> Self {
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
#[derive(Debug, Default, Clone)]
pub struct TreeState {
    pub expanded_nodes: HashMap<ElementId, bool>,
    pub selected_nodes: HashMap<ElementId, bool>,
    pub checked_nodes: HashMap<ElementId, TreeCheckedState>,
}

impl TreeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the expanded state of a node.
    pub fn set_expanded(&mut self, id: &ElementId, expanded: bool) {
        self.expanded_nodes.insert(id.clone(), expanded);
    }

    /// Check if a node is expanded.
    pub fn is_expanded(&self, id: &ElementId) -> bool {
        self.expanded_nodes.get(id).copied().unwrap_or(false)
    }

    /// Set the selected state of a node.
    pub fn set_selected(&mut self, id: &ElementId, selected: bool) {
        self.selected_nodes.insert(id.clone(), selected);
    }

    /// Check if a node is selected.
    pub fn is_selected(&self, id: &ElementId) -> bool {
        self.selected_nodes.get(id).copied().unwrap_or(false)
    }

    /// Set the checked state of a node.
    pub fn set_checked(&mut self, id: &ElementId, checked: TreeCheckedState) {
        self.checked_nodes.insert(id.clone(), checked);
    }

    /// Get the checked state of a node.
    pub fn get_checked(&self, id: &ElementId) -> TreeCheckedState {
        self.checked_nodes
            .get(id)
            .copied()
            .unwrap_or(TreeCheckedState::Unchecked)
    }

    /// Toggle the expanded state of a node.
    pub fn toggle_expanded(&mut self, id: &ElementId) {
        let current = self.is_expanded(id);
        self.set_expanded(id, !current);
    }

    /// Clear all selected nodes.
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    /// Get all selected node IDs.
    pub fn selected_ids(&self) -> impl Iterator<Item = &ElementId> {
        self.selected_nodes.keys()
    }

    /// Get all checked node IDs.
    pub fn checked_ids(&self) -> impl Iterator<Item = (&ElementId, &TreeCheckedState)> {
        self.checked_nodes.iter()
    }

    /// Get all expanded node IDs.
    pub fn expanded_nodes(&self) -> impl Iterator<Item = (&ElementId, &bool)> {
        self.expanded_nodes.iter()
    }
}

/// Event emitted by tree interactions.
#[derive(Debug, Clone)]
pub enum TreeEvent {
    /// A node was clicked.
    Click(ElementId),
    /// A node was double-clicked.
    DoubleClick(ElementId),
    /// A node's expansion state changed.
    ToggleExpand(ElementId),
    /// A node's selection changed.
    Select(ElementId),
    /// A node's checked state changed (for checkbox mode).
    Check(ElementId, TreeCheckedState),
    /// A node was dropped onto another node.
    Drop {
        dragged_id: ElementId,
        target_id: ElementId,
        position: DropPosition,
    },
}

/// Position where a node is dropped relative to the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPosition {
    /// Dropped above the target node.
    Before,
    /// Dropped below the target node.
    After,
    /// Dropped onto the target node (as a child).
    On,
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

/// Flattened tree node for virtualized rendering.
///
/// This is used internally by the Tree component to render only visible nodes.
#[derive(Debug, Clone)]
pub struct FlatTreeNode<T: TreeNodeData = ArcTreeNode> {
    /// The original node ID.
    pub id: ElementId,
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
    expanded_ids: &HashMap<ElementId, bool>,
    include_hidden: bool,
) -> Vec<FlatTreeNode<T>> {
    let mut result = Vec::new();
    flatten_tree_recursive(nodes, expanded_ids, 0, &mut result, include_hidden, 0);
    result
}

fn flatten_tree_recursive<T: TreeNodeData>(
    nodes: &[TreeNode<T>],
    expanded_ids: &HashMap<ElementId, bool>,
    depth: usize,
    result: &mut Vec<FlatTreeNode<T>>,
    include_hidden: bool,
    start_index: usize,
) -> usize {
    let mut index = start_index;

    for node in nodes {
        let is_expanded = expanded_ids.get(&node.id).copied().unwrap_or(node.expanded);

        // If not expanded and not including hidden nodes, skip this node's children
        if !is_expanded && !include_hidden && !node.expanded {
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
