//! Headless `tree_item` — a single row in a `tree`. Pure data.

use std::sync::Arc;
use std::time::Instant;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, SharedString, Stateful,
    Window,
};

pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut gpui::Window, &mut App) + Send + Sync>;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
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

impl std::fmt::Display for TreeNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

/// Maximum delay between two clicks to be considered a
/// double-click. Mirrors the platform's `GetDoubleClickTime`
/// on Windows / `NSWindow.doubleClickInterval` on macOS; 300ms
/// is the typical value used in editor UIs.
pub const DOUBLE_CLICK_THRESHOLD: std::time::Duration = std::time::Duration::from_millis(300);

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
    /// Multi-select mode: render a checkbox between the chevron
    /// and the label (WinUI `TreeView` with
    /// `SelectionMode="Multiple"`). The checkbox's own click fires
    /// `on_check` (falling back to `on_click` when unset) and
    /// does NOT bubble to the row body.
    pub checkbox: bool,
    /// Multi-select membership for this node (drives the
    /// checkbox's checked state).
    pub checked: bool,
    /// Tri-state hint for parent rows: some (but not all)
    /// descendants are checked. Paints a mixed checkbox.
    pub indeterminate: bool,
    pub focus_handle: FocusHandle,
    /// Fired on a single click on the row body (not the chevron).
    /// A double-click fires `on_double_click` instead and
    /// suppresses `on_click`.
    pub on_click: Option<ClickCallback>,
    /// Fired on a single click of the chevron — and on a
    /// double-click of the row body, so users can collapse /
    /// expand without aiming at the small chevron target.
    pub on_toggle: Option<ClickCallback>,
    /// Fired when the multi-select checkbox itself is clicked.
    /// When unset the renderer falls back to `on_click`.
    pub on_check: Option<ClickCallback>,
    /// Optional double-click callback. When unset the renderer
    /// falls back to firing `on_toggle` on double-click.
    pub on_double_click: Option<ClickCallback>,
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
        checkbox: false,
        checked: false,
        indeterminate: false,
        focus_handle: cx.focus_handle(),
        on_click: None,
        on_toggle: None,
        on_check: None,
        on_double_click: None,
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
    /// Render a multi-select checkbox on this row (see
    /// [`TreeItemProps::checkbox`]).
    pub fn checkbox(mut self, v: bool) -> Self {
        self.checkbox = v;
        self
    }
    /// Multi-select membership for this row.
    pub fn checked(mut self, v: bool) -> Self {
        self.checked = v;
        self
    }
    /// Tri-state hint — some descendants are checked.
    pub fn indeterminate(mut self, v: bool) -> Self {
        self.indeterminate = v;
        self
    }
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut gpui::Window, &mut App),
    {
        self.on_click = Some(Arc::new(f));
        self
    }
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut gpui::Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(f));
        self
    }
    /// Set the multi-select checkbox's click handler. When unset,
    /// clicking the checkbox falls back to `on_click`.
    pub fn on_check<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut gpui::Window, &mut App),
    {
        self.on_check = Some(Arc::new(f));
        self
    }
    pub fn on_double_click<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut gpui::Window, &mut App),
    {
        self.on_double_click = Some(Arc::new(f));
        self
    }
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    /// Render the tree item using the registered
    /// `TreeItemRenderer`. The renderer builds the full visual
    /// (bg / padding / indent / chevron / hover / id / on_click)
    /// and wires double-click detection using
    /// `window.use_keyed_state` keyed by the row's `id`. The
    /// headless layer only adds `track_focus` on top — the
    /// focus handle is headless state that the renderer has
    /// no business mutating.
    pub fn render(self, cx: &mut App, window: &mut Window) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::TreeItem as TreeItemMarker;
        use crate::renderer::tree_item::TreeItemRenderer;

        // Clone the renderer Arc to release the immutable
        // borrow of `cx` from `renderer_arc` before we call
        // `compose` (which needs `cx` mutably for `use_keyed_state`).
        let r: Arc<dyn TreeItemRenderer> = cx
            .renderer_arc::<TreeItemMarker, dyn TreeItemRenderer>()
            .expect("TreeItemRenderer registered")
            .clone();
        let mut div = r.compose(&self, cx, window);
        div = div.track_focus(&self.focus_handle);
        div
    }
}

/// Re-export the helper used by the renderer's double-click
/// tracker so both default and brutalism renderers can use the
/// same threshold without duplicating the constant.
pub use DOUBLE_CLICK_THRESHOLD as double_click_threshold;

/// Keyed-state payload used by the renderers to track the last
/// click on a given row. `None` means "no prior click".
#[derive(Clone, Default)]
pub struct LastClick(pub Option<Instant>);

impl LastClick {
    pub fn within(&self, threshold: std::time::Duration) -> bool {
        match self.0 {
            Some(t) => t.elapsed() <= threshold,
            None => false,
        }
    }
    pub fn stamp_now() -> Self {
        Self(Some(Instant::now()))
    }
}
