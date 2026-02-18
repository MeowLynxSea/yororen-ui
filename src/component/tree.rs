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

use std::sync::Arc;

use gpui::{
    div, ClickEvent, Div, ElementId, ListAlignment, ListSizingBehavior,
    IntoElement, ListState, ParentElement, Pixels,
    RenderOnce, StatefulInteractiveElement, Styled, Window, list, px,
};

use crate::component::{ClickCallback, ElementCallback, ElementClickCallback};
use crate::component::ElementMouseDownCallback;

use super::tree_data::{
    ArcTreeNode, FlatTreeNode, SelectionMode, TreeCheckedState, TreeNode, TreeNodeData, TreeState,
    flatten_tree,
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
    element_id: ElementId,
    base: Div,
    state: TreeState,
    nodes: Vec<TreeNode>,
    flattened: Vec<FlatTreeNode>,
    selection_mode: SelectionMode,
    show_checkbox: bool,
    draggable: bool,
    indent: Pixels,
    row_height: Pixels,
    virtualized: bool,
    list_state: Option<ListState>,
    on_click: Option<ClickCallback>,
    on_item_click: Option<ElementClickCallback>,
    on_item_context_menu: Option<ElementMouseDownCallback>,
    on_toggle_expand: Option<ElementCallback>,
    on_select: Option<ElementCallback>,
    on_check: Option<Arc<dyn Fn(&ElementId, TreeCheckedState)>>,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new(TreeState::new(), &[])
    }
}

impl Tree {
    pub fn new(state: TreeState, nodes: &[TreeNode]) -> Self {
        let mut tree = Self {
            element_id: "ui:tree".into(),
            base: div(),
            state,
            nodes: nodes.to_vec(),
            flattened: Vec::new(),
            selection_mode: SelectionMode::Multiple,
            show_checkbox: false,
            draggable: false,
            indent: px(20.),
            row_height: px(32.),
            virtualized: false,
            list_state: None,
            on_click: None,
            on_item_click: None,
            on_item_context_menu: None,
            on_toggle_expand: None,
            on_select: None,
            on_check: None,
        };
        tree.rebuild_flattened();
        tree
    }

    /// Set a stable element ID for internal keyed state.
    ///
    /// If multiple trees exist in the same window, you should provide a unique ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
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

    /// Enable virtualization for large trees.
    ///
    /// When enabled, the tree will use a virtualized list rendering,
    /// which allows for efficient scrolling of large datasets.
    pub fn virtualized(mut self, virtualized: bool) -> Self {
        self.virtualized = virtualized;
        self
    }

    /// Set the list state for virtualized rendering.
    /// This should be called when virtualized() is enabled.
    pub fn list_state(mut self, state: ListState) -> Self {
        self.list_state = Some(state);
        self
    }

    /// Set a fixed height for the virtualized tree.
    /// This is needed for the virtualized list to calculate scroll bounds.
    pub fn height(mut self, height: Pixels) -> Self {
        self.base = self.base.h(height);
        self
    }

    /// Set a click handler for the tree.
    /// The handler receives only the click event (without element ID).
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_click = Some(Arc::new(handler));
        self
    }

    /// Set a click handler that receives the clicked item's element ID.
    /// Use this when you need to know which specific item was clicked.
    pub fn on_item_click<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_item_click = Some(Arc::new(handler));
        self
    }

    /// Set a right-click (context menu) handler for individual items.
    ///
    /// This is triggered when the user right-clicks a row.
    pub fn on_item_context_menu<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId, &gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_item_context_menu = Some(Arc::new(handler));
        self
    }

    pub fn on_toggle_expand<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId),
    {
        self.on_toggle_expand = Some(Arc::new(handler));
        self
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId),
    {
        self.on_select = Some(Arc::new(handler));
        self
    }

    pub fn on_check<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId, TreeCheckedState),
    {
        self.on_check = Some(Arc::new(handler));
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
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        // If virtualized, render using gpui::list
        if self.virtualized {
            return self.render_virtualized(window, cx).into_any_element();
        }

        // Regular non-virtualized rendering
        self.render_normal(window, cx).into_any_element()
    }
}

impl Tree {
    /// Render the tree using virtualized list (gpui::list).
    fn render_virtualized(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let show_checkbox = self.show_checkbox;
        let indent = self.indent;
        let base = self.base;

        let id = self.element_id.clone();

        // Use keyed state to persist list state across renders
        let list_state = window.use_keyed_state((id.clone(), "ui:tree:list-state"), cx, |_, _| {
            self.list_state.clone().unwrap_or_else(|| {
                ListState::new(0, ListAlignment::Top, px(32.))
            })
        });

        // Recalculate flattened nodes
        let mut expanded_ids = std::collections::HashMap::new();

        // Collect expanded IDs from the persisted state
        let state_entity = window.use_keyed_state((id.clone(), "ui:tree:state"), cx, |_, _| {
            self.state.clone()
        });

        for (id, expanded) in state_entity.read(cx).expanded_nodes() {
            expanded_ids.insert(id.clone(), *expanded);
        }

        // Also check the nodes' own expanded field
        fn collect_expanded<T: super::tree_data::TreeNodeData>(
            nodes: &[super::tree_data::TreeNode<T>],
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

        // Recalculate flattened using current nodes
        let flattened = flatten_tree(&self.nodes, &expanded_ids, false);
        let item_count = flattened.len();

        // Update list state with item count.
        //
        // IMPORTANT: Avoid calling `reset` unconditionally during render.
        // `reset` clears the scroll offset, which effectively prevents scrolling
        // when the view re-renders in response to scroll events.
        list_state.update(cx, |state, _cx| {
            let old_count = state.item_count();
            if old_count != item_count {
                state.splice(0..old_count, item_count);
            }
        });

        let state_snapshot: TreeState = state_entity.read(cx).clone();
        let on_item_click = self.on_item_click;
        let on_item_context_menu = self.on_item_context_menu;
        let selection_mode = self.selection_mode;
        let _on_toggle_expand = self.on_toggle_expand;
        let _on_select = self.on_select;

        // Clone for use in closures that may be called multiple times
        let state_entity_for_toggle = state_entity.clone();
        let state_entity_for_select = state_entity.clone();
        let on_item_click_clone = on_item_click.clone();
        let on_item_context_menu_clone = on_item_context_menu.clone();

        // Create the virtualized list
        let _node_id = self.element_id.clone();
        let list = list(list_state.read(cx).clone(), move |ix, _window, _cx| {
            let node = &flattened[ix];
            let node_id = node.id.clone();
            let is_selected = state_snapshot.is_selected(&node_id);

            let label_text = node.data.label().to_string();
            let disabled = node.data.disabled;
            let has_children = node.has_children;
            let expanded = state_snapshot.is_expanded(&node_id);

            let icon_path = node.data.icon.clone().map(super::Icon::new);

            let row_id: ElementId = (node_id.clone(), "ui:tree:row").into();

            let mut row = super::tree_item::tree_item(row_id)
                .depth(node.depth)
                .indent(indent)
                .selected(is_selected)
                .disabled(disabled)
                .has_children(has_children)
                .expanded(expanded)
                .show_checkbox(show_checkbox)
                .label(super::label(label_text).ellipsis(true));

            if let Some(icon) = icon_path {
                row = row.icon(icon);
            }

            if has_children && !disabled {
                let state_entity = state_entity_for_toggle.clone();
                row = row.on_click({
                    let node_id = node_id.clone();
                    move |_ev, window, cx| {
                        state_entity.update(cx, |state, _cx| {
                            let expanded_now = state.is_expanded(&node_id);
                            state.set_expanded(&node_id, !expanded_now);
                        });
                        window.refresh();
                    }
                });
            }

            if !disabled {
                let state_entity = state_entity_for_select.clone();
                let on_item_click = on_item_click_clone.clone();
                row = row.on_click({
                    let node_id = node_id.clone();
                    move |ev, window, cx| {
                        state_entity.update(cx, |state, _cx| {
                            match selection_mode {
                                SelectionMode::Single => {
                                    state.clear_selection();
                                    state.set_selected(&node_id, true);
                                }
                                SelectionMode::Multiple => {
                                    let selected = state.is_selected(&node_id);
                                    state.set_selected(&node_id, !selected);
                                }
                                SelectionMode::None => {}
                            }
                        });

                        if let Some(handler) = &on_item_click {
                            handler(&node_id, ev, window, cx);
                        }

                        window.refresh();
                    }
                });
            }

            if !disabled {
                let on_item_context_menu = on_item_context_menu_clone.clone();
                row = row.on_context_menu({
                    let node_id = node_id.clone();
                    move |ev, window, cx| {
                        if let Some(handler) = &on_item_context_menu {
                            handler(&node_id, ev, window, cx);
                        }
                    }
                });
            }

            super::virtual_row(node_id.clone())
                .child(row)
                .into_any_element()
        })
        // NOTE: For scrollable lists we want the list to size itself from the
        // available space (i.e. the container's height), not infer its height
        // from all items.
        .with_sizing_behavior(ListSizingBehavior::Auto)
        .w_full()
        .h_full()
        .min_h_0()
        .flex_grow();

        // Preserve styling set on the `Tree` itself (e.g. `.height(px(...))`).
        // This wrapper provides the bounded height needed for the virtualized
        // list to establish a scroll viewport.
        // Note: rounded and padding styles set via Styled are NOT preserved in virtualized mode.
        // If you need these styles in virtualized mode, wrap the tree in a parent div instead.
        base.flex().flex_col().w_full().h_full().min_h_0().flex_grow().child(list)
    }

    /// Render the tree using normal flex layout.
    fn render_normal(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let show_checkbox = self.show_checkbox;
        let indent = self.indent;
        let draggable = self.draggable;

        // Recalculate flattened nodes using the current nodes.
        // This is necessary because the Tree may be reconstructed with new nodes
        // on each render (e.g., when the underlying data changes), but the expansion
        // state is persisted via keyed state.
        let mut expanded_ids = std::collections::HashMap::new();
        let id = self.element_id;

        // Tree needs internal state to be updatable from click handlers.
        // Store TreeState in a keyed entity so closures can call `update`.
        let state_entity = window.use_keyed_state((id.clone(), "ui:tree:state"), cx, |_, _| {
            self.state.clone()
        });

        // Collect expanded IDs from the persisted state
        for (id, expanded) in state_entity.read(cx).expanded_nodes() {
            expanded_ids.insert(id.clone(), *expanded);
        }

        // Also check the nodes' own expanded field
        fn collect_expanded<T: super::tree_data::TreeNodeData>(
            nodes: &[super::tree_data::TreeNode<T>],
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

        // Recalculate flattened using current nodes
        let flattened = flatten_tree(&self.nodes, &expanded_ids, false);

        // Get the current state snapshot for rendering
        let state_snapshot: TreeState = state_entity.read(cx).clone();

        let on_item_click = self.on_item_click;
        let on_click = self.on_click;
        let on_item_context_menu = self.on_item_context_menu;
        let on_toggle_expand = self.on_toggle_expand;
        let selection_mode = self.selection_mode;
        let on_select = self.on_select;

        // NOTE: `Tree` is the stateful container (expanded + selection).
        // `tree_item` is the presentational row (indent + disclosure + icon + label).
        // Wiring click handling here keeps `TreeItem` generic and reusable.

        self.base
            .flex()
            .flex_col()
            .gap_1()
            .children(flattened.into_iter().map(move |node| {
                let node_id = node.id.clone();
                let is_selected = state_snapshot.is_selected(&node_id);

                let label_text = node.data.label().to_string();
                let disabled = node.data.disabled;
                let has_children = node.has_children;
                let expanded = state_snapshot.is_expanded(&node_id);

                let on_item_click = on_item_click.clone();
                let on_click = on_click.clone();
                let on_item_context_menu = on_item_context_menu.clone();
                let on_toggle_expand = on_toggle_expand.clone();
                let on_select = on_select.clone();

                // Only support file icon paths for the default ArcTreeNode for now.
                let icon_path = node.data.icon.clone().map(super::Icon::new);

                let row_id: ElementId = (node_id.clone(), "ui:tree:row").into();

                let mut row = super::tree_item::tree_item(row_id)
                    .depth(node.depth)
                    .indent(indent)
                    .selected(is_selected)
                    .disabled(disabled)
                    .has_children(has_children)
                    .expanded(expanded)
                    .show_checkbox(show_checkbox)
                    .label(super::label(label_text).ellipsis(true));

                if let Some(icon) = icon_path {
                    row = row.icon(icon);
                }

                // Expand/collapse toggle: currently handled by treating the disclosure area
                // as a normal click target. TreeItem does not have a dedicated handler API.
                if has_children && !disabled {
                    row = row.on_click({
                        let node_id = node_id.clone();
                        let state_entity = state_entity.clone();
                        move |_ev, window, cx| {
                            state_entity.update(cx, |state, _cx| {
                                let expanded_now = state.is_expanded(&node_id);
                                state.set_expanded(&node_id, !expanded_now);
                            });

                            if let Some(handler) = &on_toggle_expand {
                                handler(&node_id);
                            }

                            window.refresh();
                        }
                    });
                }

                // Selection click on the whole row.
                if !disabled {
                    row = row.on_click({
                        let node_id = node_id.clone();
                        let state_entity = state_entity.clone();
                        move |ev, window, cx| {
                            state_entity.update(cx, |state, _cx| {
                                match selection_mode {
                                    SelectionMode::Single => {
                                        state.clear_selection();
                                        state.set_selected(&node_id, true);
                                    }
                                    SelectionMode::Multiple => {
                                        let selected = state.is_selected(&node_id);
                                        state.set_selected(&node_id, !selected);
                                    }
                                    SelectionMode::None => {}
                                }
                            });

                            if let Some(handler) = &on_item_click {
                                handler(&node_id, ev, window, cx);
                            }
                            if let Some(handler) = &on_click {
                                handler(ev, window, cx);
                            }

                            if let Some(handler) = &on_select {
                                handler(&node_id);
                            }

                            window.refresh();
                        }
                    });
                }

                if !disabled {
                    row = row.on_context_menu({
                        let node_id = node_id.clone();
                        move |ev, window, cx| {
                            if let Some(handler) = &on_item_context_menu {
                                handler(&node_id, ev, window, cx);
                            }
                        }
                    });
                }

                // Draggable is currently a Tree-level capability; keep it for future work.
                // This field is retained for API stability.
                let _ = draggable;

                super::virtual_row(node_id.clone()).child(row)
            }))
    }
}

/// Builder function for creating tree nodes with ArcTreeNode data.
pub fn tree_node_data(label: impl Into<String>) -> ArcTreeNode {
    ArcTreeNode::new(label)
}
