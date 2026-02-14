//! Callback type definitions for UI components.
//!
//! This module provides unified callback type aliases to ensure consistency
//! across all components. All callbacks use `Arc<dyn Fn>` for optimal
//! performance and flexibility in Rust.

use std::sync::Arc;

use gpui::{App, ClickEvent, ElementId, Window};

/// Callback for click events.
///
/// # Parameters
/// - `&ClickEvent` - The click event data
/// - `&mut Window` - The window context
/// - `&mut App` - The application context
pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Callback for click events with element identifier.
///
/// # Parameters
/// - `&ElementId` - The identifier of the clicked element
/// - `&ClickEvent` - The click event data
/// - `&mut Window` - The window context
/// - `&mut App` - The application context
pub type ElementClickCallback =
    Arc<dyn Fn(&ElementId, &ClickEvent, &mut Window, &mut App)>;

/// Callback for hover state changes.
///
/// # Parameters
/// - `bool` - Whether the element is hovered
/// - `&mut Window` - The window context
/// - `&mut App` - The application context
pub type HoverCallback = Arc<dyn Fn(bool, &mut Window, &mut App)>;

/// Callback for toggle events (e.g., checkbox, switch).
///
/// # Parameters
/// - `bool` - The new toggle state
/// - `&ClickEvent` - The click event data (optional, can be None for programmatic changes)
/// - `&mut Window` - The window context
/// - `&mut App` - The application context
pub type ToggleCallback =
    Arc<dyn Fn(bool, Option<&ClickEvent>, &mut Window, &mut App)>;

/// Callback for value changes with generic value type.
///
/// # Parameters
/// - `T` - The new value
/// - `&mut Window` - The window context
/// - `&mut App` - The application context
pub type ChangeCallback<T> = Arc<dyn Fn(T, &mut Window, &mut App)>;

/// Callback for value changes with event information.
///
/// # Parameters
/// - `T` - The new value
/// - `&ClickEvent` - The click event data
/// - `&mut Window` - The window context
/// - `&mut App` - The application context
pub type ChangeWithEventCallback<T> =
    Arc<dyn Fn(T, &ClickEvent, &mut Window, &mut App)>;

/// Callback for generic element identifier events.
///
/// # Parameters
/// - `&ElementId` - The identifier of the element
pub type ElementCallback = Arc<dyn Fn(&ElementId)>;

/// Callback for window events.
///
/// # Parameters
/// - `&mut Window` - The window context
/// - `&mut App` - The application context
pub type WindowCallback = Arc<dyn Fn(&mut Window, &mut App)>;

/// Callback for generic events.
///
/// # Parameters
/// - `T` - The event data
pub type EventCallback<T> = Arc<dyn Fn(T)>;
