//! Headless `tree` — the data structure of a tree (parent/child
//! relationships) and a current selection + expansion set. The
//! visual lives in the renderer; callers iterate `children(id)` to
//! produce rows.
//!
//! ## Selection modes
//!
//! - [`TreeSelectionMode::Single`] (default) — `selected` holds
//!   the one selected node (classic TreeView).
//! - [`TreeSelectionMode::Multiple`] — `selected_ids` holds the
//!   selection set. Rows typically render a checkbox (see
//!   `TreeItemProps::checkbox`); helpers on [`TreeData`] cover
//!   the common multi-select gestures (toggle one, select a
//!   visible range, toggle a whole subtree).
//! - [`TreeSelectionMode::None`] — selection disabled.

use crate::renderer::RendererContext;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use gpui::{App, Div, ElementId, InteractiveElement, SharedString, Stateful};

use super::tree_item::TreeNodeId;

/// How many nodes a tree allows in its selection at once.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeSelectionMode {
    /// Clicking / checking does nothing selection-wise.
    None,
    /// Exactly one selected node (`TreeProps::selected`).
    #[default]
    Single,
    /// Any number of selected nodes (`TreeProps::selected_ids`);
    /// rows render checkboxes when the caller opts in per row.
    Multiple,
}

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
    /// Return a depth-first pre-order list of visible nodes.
    ///
    /// A node is visible iff every ancestor is in `expanded`.
    /// The returned `Vec<(TreeNodeId, usize depth)>` is in render
    /// order — iterate and emit one `tree_item` per entry.
    pub fn flatten(&self, expanded: &BTreeSet<TreeNodeId>) -> Vec<(TreeNodeId, usize)> {
        let mut out = Vec::new();
        for root in &self.roots {
            self.flatten_inner(root, 0, expanded, &mut out);
        }
        out
    }
    fn flatten_inner(
        &self,
        id: &TreeNodeId,
        depth: usize,
        expanded: &BTreeSet<TreeNodeId>,
        out: &mut Vec<(TreeNodeId, usize)>,
    ) {
        out.push((id.clone(), depth));
        if expanded.contains(id)
            && let Some(kids) = self.children.get(id)
        {
            for child in kids {
                self.flatten_inner(child, depth + 1, expanded, out);
            }
        }
    }

    /// `id` plus every transitive descendant, in depth-first
    /// pre-order. Multi-select trees use this to toggle a whole
    /// subtree from a parent row's checkbox.
    pub fn subtree_ids(&self, id: &TreeNodeId) -> Vec<TreeNodeId> {
        let mut out = vec![id.clone()];
        self.push_descendants(id, &mut out);
        out
    }
    fn push_descendants(&self, id: &TreeNodeId, out: &mut Vec<TreeNodeId>) {
        if let Some(kids) = self.children.get(id) {
            for child in kids {
                out.push(child.clone());
                self.push_descendants(child, out);
            }
        }
    }

    /// The inclusive range of *visible* nodes between `from` and
    /// `to` in flattened render order — the shift-click gesture
    /// for multi-select trees. Returns an empty set when either
    /// endpoint is not currently visible.
    pub fn select_range(
        &self,
        expanded: &BTreeSet<TreeNodeId>,
        from: &TreeNodeId,
        to: &TreeNodeId,
    ) -> BTreeSet<TreeNodeId> {
        let visible = self.flatten(expanded);
        let ix_of = |n: &TreeNodeId| visible.iter().position(|(id, _)| id == n);
        let (Some(a), Some(b)) = (ix_of(from), ix_of(to)) else {
            return BTreeSet::new();
        };
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        visible[lo..=hi].iter().map(|(id, _)| id.clone()).collect()
    }
}

#[derive(Clone)]
pub struct TreeProps {
    pub id: ElementId,
    pub data: TreeData,
    pub expanded: BTreeSet<TreeNodeId>,
    /// Single-selection anchor (`Single` mode).
    pub selected: Option<TreeNodeId>,
    /// Multi-selection set (`Multiple` mode). Renderers read this
    /// only for container-level hints (e.g. "has any selection");
    /// per-row visuals come from `TreeItemProps::checked`.
    pub selected_ids: BTreeSet<TreeNodeId>,
    pub selection_mode: TreeSelectionMode,
}

pub fn tree(id: impl Into<ElementId>, _cx: &mut App) -> TreeProps {
    TreeProps {
        id: id.into(),
        data: TreeData::new(),
        expanded: BTreeSet::new(),
        selected: None,
        selected_ids: BTreeSet::new(),
        selection_mode: TreeSelectionMode::Single,
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
    pub fn selected(mut self, id: impl Into<Option<TreeNodeId>>) -> Self {
        self.selected = id.into();
        self
    }
    /// Set the multi-selection set wholesale (`Multiple` mode).
    pub fn selected_ids(mut self, ids: BTreeSet<TreeNodeId>) -> Self {
        self.selected_ids = ids;
        self
    }
    /// Add one node to the multi-selection set (`Multiple` mode).
    pub fn selected_id(mut self, id: impl Into<TreeNodeId>) -> Self {
        self.selected_ids.insert(id.into());
        self
    }
    /// Override the selection mode. `.multi_select()` /
    /// `.single_select()` are the common shorthands.
    pub fn selection_mode(mut self, mode: TreeSelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }
    /// Switch the tree to `Multiple` selection.
    pub fn multi_select(self) -> Self {
        self.selection_mode(TreeSelectionMode::Multiple)
    }
    /// Switch the tree to `Single` selection (the default).
    pub fn single_select(self) -> Self {
        self.selection_mode(TreeSelectionMode::Single)
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the tree container through the registered `TreeRenderer`.
    ///
    /// Tree rows are added by the caller as children after `.render(cx)`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let r = cx
            .renderer_arc::<crate::renderer::markers::Tree, dyn crate::renderer::tree::TreeRenderer>()
            .expect("TreeRenderer registered");
        r.compose(&self, cx)
    }
}

/// Re-export the helper type used by `tree_item` so callers don't
/// have to know the headless module layout.
pub use super::tree_item::TreeNodeId as _TreeNodeId;
/// Convenience: a node id from a static `&str` or a `SharedString`.
pub fn node_id(s: impl Into<SharedString>) -> TreeNodeId {
    TreeNodeId(s.into())
}
