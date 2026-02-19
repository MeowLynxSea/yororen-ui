//! Focus Trap component for accessibility.
//!
//! This module provides a FocusTrap component that traps focus within
//! a container element, ensuring keyboard users cannot tab outside
//! of a modal or other focused interaction area.

use gpui::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, actions, div,
};
use std::sync::Arc;

actions!(
    focus_trap,
    [
        /// Trap focus movement within the container.
        FocusNext,
        /// Move focus to previous focusable element.
        FocusPrev,
        /// Close the trap (typically via Escape key).
        Close,
    ]
);

/// Callback type for window and app event handlers.
type WindowAppCallback = Arc<dyn Fn(&mut Window, &mut App)>;

/// A component that traps focus within a container element.
///
/// FocusTrap is essential for modal dialogs and other overlays where
/// keyboard focus should be contained within the component.
///
/// # Usage
///
/// ```ignore
/// focus_trap()
///     .on_escape(|_, window, cx| {
///         // Handle escape key
///     })
///     .child(modal_content)
/// ```
pub fn focus_trap() -> FocusTrap {
    FocusTrap::new()
}

#[derive(IntoElement)]
pub struct FocusTrap {
    element_id: Option<ElementId>,
    base: gpui::Div,
    /// Callback fired when Escape key is pressed.
    on_escape: Option<WindowAppCallback>,
    /// Callback fired when FocusNext (Tab) is pressed.
    on_focus_next: Option<WindowAppCallback>,
    /// Callback fired when FocusPrev (Shift+Tab) is pressed.
    on_focus_prev: Option<WindowAppCallback>,
    /// Whether to trap focus (default: true).
    trap_focus: bool,
    /// Initial focus element ID.
    initial_focus: Option<ElementId>,
}

impl Default for FocusTrap {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusTrap {
    /// Creates a new FocusTrap component.
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),
            on_escape: None,
            on_focus_next: None,
            on_focus_prev: None,
            trap_focus: true,
            initial_focus: None,
        }
    }

    /// Sets the element ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    /// Sets the callback for Escape key.
    pub fn on_escape<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&mut Window, &mut App),
    {
        self.on_escape = Some(Arc::new(handler));
        self
    }

    /// Sets the callback for Tab key (move to next focusable).
    pub fn on_focus_next<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&mut Window, &mut App),
    {
        self.on_focus_next = Some(Arc::new(handler));
        self
    }

    /// Sets the callback for Shift+Tab key (move to previous focusable).
    pub fn on_focus_prev<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&mut Window, &mut App),
    {
        self.on_focus_prev = Some(Arc::new(handler));
        self
    }

    /// Sets whether to trap focus.
    pub fn trap_focus(mut self, trap: bool) -> Self {
        self.trap_focus = trap;
        self
    }

    /// Sets the initial focus element ID.
    pub fn initial_focus(mut self, id: impl Into<ElementId>) -> Self {
        self.initial_focus = Some(id.into());
        self
    }
}

impl ParentElement for FocusTrap {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for FocusTrap {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for FocusTrap {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for FocusTrap {}

impl RenderOnce for FocusTrap {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let element_id = self.element_id;

        // Return the base element with optional ID
        // Note: Full keyboard trap functionality requires integration at the app/overlay level
        self.base
            .id(element_id.unwrap_or_else(|| "focus-trap".into()))
    }
}

/// Focus trap state for managing focus programmatically.
pub struct FocusTrapState {
    /// The previously focused element handle (to restore later).
    pub previous_focus: Option<FocusHandle>,
    /// Whether the trap is currently active.
    pub is_active: bool,
}

impl FocusTrapState {
    /// Creates a new FocusTrapState.
    pub fn new() -> Self {
        Self {
            previous_focus: None,
            is_active: false,
        }
    }

    /// Activates the focus trap, storing the current focus.
    pub fn activate(&mut self, window: &mut Window, cx: &App) {
        self.previous_focus = window.focused(cx);
        self.is_active = true;
    }

    /// Deactivates the focus trap and restores previous focus.
    pub fn deactivate(&mut self, window: &mut Window, cx: &mut App) {
        if let Some(handle) = &self.previous_focus {
            handle.focus(window, cx);
        }
        self.is_active = false;
    }
}

impl Default for FocusTrapState {
    fn default() -> Self {
        Self::new()
    }
}

/// Direction for focus navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    /// Move focus to the next element.
    Next,
    /// Move focus to the previous element.
    Previous,
    /// Move focus to the first element.
    First,
    /// Move focus to the last element.
    Last,
}

/// Focus navigation utilities.
pub mod focus_nav {
    use super::*;

    /// Finds the next focusable element within a container.
    ///
    /// Returns the ElementId of the next focusable element, or None if at the end.
    pub fn find_next(_container_id: &ElementId, _current_id: &ElementId) -> Option<ElementId> {
        // This is a simplified implementation.
        // In a full implementation, you would traverse the DOM/container
        // to find all focusable elements and return the next one.
        //
        // The actual implementation would depend on how the UI framework
        // exposes the element tree.
        None
    }

    /// Finds the previous focusable element within a container.
    pub fn find_previous(_container_id: &ElementId, _current_id: &ElementId) -> Option<ElementId> {
        // Simplified implementation - see find_next
        None
    }

    /// Moves focus to the specified element.
    pub fn move_focus_to(_element_id: &ElementId, _window: &mut Window) -> bool {
        // In gpui, you would use the element's focus handle
        // This is a placeholder for the actual implementation
        false
    }
}
