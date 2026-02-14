//! Tree component for rendering hierarchical data.
//!
//! # Design Rationale
//!
//! The Tree component uses a different construction pattern from other components:
//! `tree(state, nodes)` requires both `TreeState` and `&[TreeNode]` as constructor parameters.
//!
//! This design choice was made because:
//! - **State and data are fundamental**: Unlike styling properties that can be added later,
//!   the tree's data structure (`TreeNode`) and state management (`TreeState`) are core to its
//!   functionality and must exist at creation time
//! - **Consistency with data-driven components**: Tree represents hierarchical data, similar to
//!   how a data table or list view requires data at initialization
//! - **Avoiding partial states**: Allowing creation without data could lead to inconsistent states
//!   where the component exists but has no content
//!
//! If you need to create a tree dynamically, consider passing an empty slice initially and
//! populating it later through the state management.

use gpui::{
    div, ClickEvent, Div, ElementId,
    IntoElement, ParentElement, Pixels,
    RenderOnce, Styled, px,
};

use super::tree_data::{
    FlatTreeNode, SelectionMode, TreeCheckedState, TreeNode,
    TreeState, flatten_tree, ArcTreeNode,
};

/// Creates a new tree component.
///
/// # Example
///
/// ```rust
/// let state = TreeState::new();
/// let nodes = vec![
///     TreeNode::new("root")
///         .children(vec![TreeNode::new("child")])
/// ];
///
/// tree(state, &nodes)
///     .selection_mode(SelectionMode::Single)
/// ```
pub fn tree(state: TreeState, nodes: &[TreeNode]) -> Tree {
    Tree::new(state, nodes)
}

/// The main tree view component.
#[derive(IntoElement)]
pub struct Tree {
    base: Div,
    state: TreeState,
    nodes: Vec<TreeNode>,
    flattened: Vec<FlatTreeNode>,
    selection_mode: SelectionMode,
    show_checkbox: bool,
    draggable: bool,
    indent: Pixels,
    row_height: Pixels,
    on_click: Option<Box<dyn Fn(&ElementId, &ClickEvent, &mut gpui::Window, &mut gpui::App)>>,
    on_toggle_expand: Option<Box<dyn Fn(&ElementId)>>,
    on_select: Option<Box<dyn Fn(&ElementId)>>,
    on_check: Option<Box<dyn Fn(&ElementId, TreeCheckedState)>>,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new(TreeState::new(), &[])
    }
}

impl Tree {
    pub fn new(state: TreeState, nodes: &[TreeNode]) -> Self {
        let mut tree = Self {
            base: div(),
            state,
            nodes: nodes.to_vec(),
            flattened: Vec::new(),
            selection_mode: SelectionMode::Multiple,
            show_checkbox: false,
            draggable: false,
            indent: px(20.),
            row_height: px(32.),
            on_click: None,
            on_toggle_expand: None,
            on_select: None,
            on_check: None,
        };
        tree.rebuild_flattened();
        tree
    }

    fn rebuild_flattened(&mut self) {
        let mut expanded_ids = std::collections::HashMap::new();
        for (id, expanded) in &self.state.expanded_nodes {
            expanded_ids.insert(id.clone(), *expanded);
        }

        fn collect_expanded(
            nodes: &[TreeNode],
            expanded: &mut std::collections::HashMap<ElementId, bool>,
        ) {
            for node in nodes {
                if node.expanded {
                    expanded.insert(node.id.clone(), true);
                }
                collect_expanded(&node.children, expanded);
            }
        }
        collect_expanded(&self.nodes, &mut expanded_ids);

        self.flattened = flatten_tree(&self.nodes, &expanded_ids, false);
    }

    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn show_checkbox(mut self, show: bool) -> Self {
        self.show_checkbox = show;
        self
    }

    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    pub fn indent(mut self, indent: Pixels) -> Self {
        self.indent = indent;
        self
    }

    pub fn row_height(mut self, height: Pixels) -> Self {
        self.row_height = height;
        self
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_expand<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId),
    {
        self.on_toggle_expand = Some(Box::new(handler));
        self
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId),
    {
        self.on_select = Some(Box::new(handler));
        self
    }

    pub fn on_check<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId, TreeCheckedState),
    {
        self.on_check = Some(Box::new(handler));
        self
    }

    pub fn toggle_expand(&mut self, id: &ElementId) {
        self.state.toggle_expanded(id);
        self.rebuild_flattened();
    }

    pub fn expand(&mut self, id: &ElementId) {
        self.state.set_expanded(id, true);
        self.rebuild_flattened();
    }

    pub fn collapse(&mut self, id: &ElementId) {
        self.state.set_expanded(id, false);
        self.rebuild_flattened();
    }

    pub fn expand_all(&mut self) {
        fn set_expanded_recursive(nodes: &[TreeNode], state: &mut TreeState) {
            for node in nodes {
                state.set_expanded(&node.id, true);
                set_expanded_recursive(&node.children, state);
            }
        }
        set_expanded_recursive(&self.nodes, &mut self.state);
        self.rebuild_flattened();
    }

    pub fn collapse_all(&mut self) {
        self.state.expanded_nodes.clear();
        self.rebuild_flattened();
    }

    pub fn select(&mut self, id: &ElementId) {
        match self.selection_mode {
            SelectionMode::Single => {
                self.state.clear_selection();
                self.state.set_selected(id, true);
            }
            SelectionMode::Multiple => {
                self.state.set_selected(id, !self.state.is_selected(id));
            }
            SelectionMode::None => {}
        }
        self.rebuild_flattened();
    }

    pub fn state(&self) -> &TreeState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut TreeState {
        &mut self.state
    }

    pub fn flattened_nodes(&self) -> &[FlatTreeNode] {
        &self.flattened
    }
}

impl ParentElement for Tree {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Tree {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Tree {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let show_checkbox = self.show_checkbox;
        let indent = self.indent;
        let draggable = self.draggable;
        let flattened = self.flattened;
        let state = self.state;

        self.base
            .flex()
            .flex_col()
            .gap_1()
            .children(flattened.into_iter().map(move |node| {
                let node_id = node.id.clone();
                let is_selected = state.is_selected(&node_id);

                super::tree_node::tree_node()
                    .node(node)
                    .show_checkbox(show_checkbox)
                    .indent(indent)
                    .selected(is_selected)
                    .draggable(draggable)
            }))
    }
}

/// Builder function for creating tree nodes with ArcTreeNode data.
pub fn tree_node_data(label: impl Into<String>) -> ArcTreeNode {
    ArcTreeNode::new(label)
}
