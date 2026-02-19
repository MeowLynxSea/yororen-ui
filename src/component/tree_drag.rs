//! Tree drag and drop functionality.
//!
//! This module provides utilities for implementing drag and drop
//! reordering in tree views.

use crate::theme::ActiveTheme;
use gpui::{ElementId, IntoElement, ParentElement, Pixels, Point, RenderOnce, Styled, div, px};

use super::tree_data::DropPosition;

/// Represents a drag operation in progress.
#[derive(Debug, Clone)]
pub struct TreeDragState {
    /// The ID of the node being dragged.
    pub dragged_id: ElementId,
    /// The current position of the drag.
    pub current_position: Point<Pixels>,
    /// The drop target (if hovering over a valid target).
    pub drop_target: Option<DropTarget>,
    /// Whether the drag is currently active.
    pub is_dragging: bool,
}

impl Default for TreeDragState {
    fn default() -> Self {
        Self {
            dragged_id: "".into(),
            current_position: Point::new(px(0.), px(0.)),
            drop_target: None,
            is_dragging: false,
        }
    }
}

/// Represents a potential drop target.
#[derive(Debug, Clone)]
pub struct DropTarget {
    /// The ID of the target node.
    pub id: ElementId,
    /// The position relative to the target node.
    pub position: DropPosition,
    /// The Y offset from the top of the target node.
    pub y_offset: Pixels,
}

/// Callback type for tree drag drop handler.
type TreeDropCallback = Box<dyn Fn(&ElementId, &ElementId, DropPosition)>;

/// Drag and drop controller for tree views.
pub struct TreeDragController {
    /// Current drag state.
    state: TreeDragState,
    /// Row height for calculations.
    _row_height: Pixels,
    /// Callback when a drop occurs.
    on_drop: Option<TreeDropCallback>,
}

impl TreeDragController {
    /// Create a new drag controller.
    pub fn new(row_height: Pixels) -> Self {
        Self {
            state: TreeDragState::default(),
            _row_height: row_height,
            on_drop: None,
        }
    }

    /// Set the drop callback.
    pub fn on_drop<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ElementId, &ElementId, DropPosition),
    {
        self.on_drop = Some(Box::new(handler));
        self
    }

    /// Start a drag operation.
    pub fn start_drag(&mut self, id: ElementId, position: Point<Pixels>) {
        self.state.dragged_id = id;
        self.state.current_position = position;
        self.state.is_dragging = true;
        self.state.drop_target = None;
    }

    /// End the drag operation and perform the drop if valid.
    pub fn end_drag(&mut self) -> Option<(ElementId, ElementId, DropPosition)> {
        if !self.state.is_dragging {
            return None;
        }

        let result = self.state.drop_target.as_ref().map(|target| {
            (
                self.state.dragged_id.clone(),
                target.id.clone(),
                target.position,
            )
        });

        if let Some((dragged_id, target_id, position)) = &result
            && let Some(handler) = &self.on_drop
        {
            handler(dragged_id, target_id, *position);
        }

        self.state = TreeDragState::default();
        result
    }

    /// Cancel the drag operation.
    pub fn cancel_drag(&mut self) {
        self.state = TreeDragState::default();
    }

    /// Get the current drag state.
    pub fn state(&self) -> &TreeDragState {
        &self.state
    }

    /// Check if currently dragging.
    pub fn is_dragging(&self) -> bool {
        self.state.is_dragging
    }

    /// Get the dragged node ID.
    pub fn dragged_id(&self) -> &ElementId {
        &self.state.dragged_id
    }

    /// Get the current drop target.
    pub fn drop_target(&self) -> Option<&DropTarget> {
        self.state.drop_target.as_ref()
    }
}

/// Drag preview element for tree items.
#[derive(IntoElement)]
pub struct TreeDragPreview {
    text: String,
    width: Pixels,
    height: Pixels,
}

impl TreeDragPreview {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            width: px(200.),
            height: px(32.),
        }
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }
}

impl RenderOnce for TreeDragPreview {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        let theme = cx.theme();

        div()
            .w(self.width)
            .h(self.height)
            .bg(theme.surface.raised)
            .border_1()
            .border_color(theme.border.default)
            .rounded_md()
            .shadow_lg()
            .px_3()
            .flex()
            .items_center()
            .text_color(theme.content.primary)
            .child(self.text)
    }
}
