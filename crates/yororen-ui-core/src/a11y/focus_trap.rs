//! Focus Trap component for accessibility.
//!
//! This module provides a `FocusTrap` component that traps focus
//! within a container element, ensuring keyboard users cannot tab
//! outside of a modal or other focused interaction area.
//!
//! # v0.5 behavior
//!
//! The `RenderOnce` impl wires three keyboard handlers:
//!
//! - **Escape** (`capture_key_down` with `keystroke.key == "escape"`)
//!   fires `on_escape` and calls `cx.stop_propagation()` so parent
//!   overlays don't double-fire.
//! - **Tab** (key == "tab" with no shift modifier) fires
//!   `on_focus_next`. The actual focus movement is left to the
//!   caller — FocusTrap doesn't know which child elements are
//!   focusable. Most apps will want to wrap FocusTrap around a
//!   container that has explicit focusable children, and the
//!   caller can use `keyboard_nav::find_next` (a11y helper)
//!   to determine the next element. If the caller doesn't
//!   implement this, focus will simply move out of the trap
//!   (gpui's default Tab behavior).
//! - **Shift+Tab** (key == "tab" with shift modifier) fires
//!   `on_focus_prev` symmetrically.
//!
//! The `FocusTrapState` helper struct provides
//! `activate` / `deactivate` methods that capture and restore the
//! focused element, so a modal can return focus to its trigger
//! when closed.

use gpui::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, actions, div,
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
pub type WindowAppCallback = Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>;

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
///
/// # Behavior
///
/// - **Escape**: fires `on_escape` and stops propagation.
/// - **Tab**: fires `on_focus_next` (the actual focus move is up
///   to the caller; if no handler is set, gpui's default Tab
///   behavior takes over).
/// - **Shift+Tab**: fires `on_focus_prev` symmetrically.
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
        }
    }

    /// Sets the element ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Sets the callback for Escape key.
    pub fn on_escape<F>(mut self, handler: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut Window, &mut App),
    {
        self.on_escape = Some(Arc::new(handler));
        self
    }

    /// Sets the callback for Tab key (move to next focusable).
    pub fn on_focus_next<F>(mut self, handler: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut Window, &mut App),
    {
        self.on_focus_next = Some(Arc::new(handler));
        self
    }

    /// Sets the callback for Shift+Tab key (move to previous focusable).
    pub fn on_focus_prev<F>(mut self, handler: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut Window, &mut App),
    {
        self.on_focus_prev = Some(Arc::new(handler));
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
        let on_escape = self.on_escape;
        let on_focus_next = self.on_focus_next;
        let on_focus_prev = self.on_focus_prev;

        // Wire keyboard handlers onto the base div. The handlers
        // are installed via `capture_key_down` so they fire even
        // when a child element has focus (which is the common
        // case for a modal).
        let mut base = self.base;
        if let Some(handler) = on_escape {
            base = base.capture_key_down(move |event: &KeyDownEvent, window, cx| {
                if event.keystroke.key.eq_ignore_ascii_case("escape") {
                    cx.stop_propagation();
                    handler(window, cx);
                }
            });
        }
        if on_focus_next.is_some() || on_focus_prev.is_some() {
            base = base.capture_key_down(move |event: &KeyDownEvent, window, cx| {
                if !event.keystroke.key.eq_ignore_ascii_case("tab") {
                    return;
                }
                let shift = event.keystroke.modifiers.shift;
                if !shift && let Some(handler) = &on_focus_next {
                    cx.stop_propagation();
                    handler(window, cx);
                } else if shift && let Some(handler) = &on_focus_prev {
                    cx.stop_propagation();
                    handler(window, cx);
                }
            });
        }
        base.id(element_id.unwrap_or_else(|| "focus-trap".into()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_focus_trap_is_unarmed() {
        let ft = FocusTrap::new();
        assert!(ft.on_escape.is_none());
        assert!(ft.on_focus_next.is_none());
        assert!(ft.on_focus_prev.is_none());
    }

    #[test]
    fn setters_update_callbacks() {
        let ft = FocusTrap::new()
            .on_escape(|_w, _c| {})
            .on_focus_next(|_w, _c| {})
            .on_focus_prev(|_w, _c| {});
        assert!(ft.on_escape.is_some());
        assert!(ft.on_focus_next.is_some());
        assert!(ft.on_focus_prev.is_some());
    }

    #[test]
    fn focus_trap_state_round_trip() {
        let mut s = FocusTrapState::new();
        assert!(!s.is_active);
        s.is_active = true;
        assert!(s.is_active);
        s.is_active = false;
        assert!(!s.is_active);
    }

    #[test]
    fn focus_direction_variants_distinct() {
        assert_ne!(FocusDirection::Next, FocusDirection::Previous);
        assert_ne!(FocusDirection::First, FocusDirection::Last);
    }
}
