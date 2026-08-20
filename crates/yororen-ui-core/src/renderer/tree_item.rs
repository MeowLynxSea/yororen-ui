//! `TreeItemRenderer` — visual contract for `TreeItem`.
//!
//! `compose` takes `&mut App, &mut Window` so the renderer can
//! mint per-row keyed state for double-click detection. The
//! renderer owns all click wiring (on_click, on_toggle,
//! on_double_click); `TreeItemProps::apply` only adds the
//! element id and focus handle.
//!
//! Inherent helpers (bg / hover_bg / selected_bg / fg / indent
//! / padding / min_height / chevron_size / radius) stay on the
//! concrete renderer type.

use std::any::Any;

use gpui::{App, Div, Stateful, Window};

use crate::headless::tree_item::TreeItemProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct TreeItemRenderState {
    pub selected: bool,
    pub expanded: bool,
    pub depth: u8,
    pub is_leaf: bool,
    /// Multi-select checkbox membership (`checkbox` rows only).
    pub checked: bool,
    /// Tri-state hint — some descendants are checked.
    pub indeterminate: bool,
}

pub trait TreeItemRenderer: Any + Send + Sync {
    /// Build the full visual + click wiring for a tree row.
    /// Returns `Stateful<Div>` because the row carries an
    /// `on_click` handler (select) and may carry a
    /// double-click handler (toggle / on_double_click).
    fn compose(&self, props: &TreeItemProps, cx: &mut App, window: &mut Window) -> Stateful<Div>;
}
