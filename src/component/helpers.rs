//! Helper functions for UI components.
//!
//! This module provides common utility functions used across multiple components
//! to reduce code duplication.

use gpui::{App, ElementId, Entity, Window};

use crate::theme::Theme;

/// Input style configuration for input components.
///
/// This struct holds the computed style values for input components
/// like TextInput, NumberInput, Select, etc.
#[derive(Clone, Debug)]
pub struct InputStyle {
    /// The background color of the input.
    pub bg: gpui::Hsla,
    /// The border color of the input.
    pub border: gpui::Hsla,
    /// The focus border color of the input.
    pub focus_border: gpui::Hsla,
    /// The text color of the input.
    pub text_color: gpui::Hsla,
}

/// Computes the input style based on theme and component properties.
///
/// This function consolidates the common style resolution logic found in
/// TextInput, NumberInput, Select, and other input components.
///
/// # Parameters
/// - `theme` - The application theme
/// - `disabled` - Whether the input is disabled
/// - `bg_color` - Optional custom background color overrides
/// - `border_color` - Optional custom border color overrides
/// - `focus_border_color` - Optional custom focus border color
/// - `text_color` - Optional custom text color
///
/// # Returns
/// An `InputStyle` struct containing the computed colors.
pub fn compute_input_style(
    theme: &Theme,
    disabled: bool,
    bg_color: Option<gpui::Hsla>,
    border_color: Option<gpui::Hsla>,
    focus_border_color: Option<gpui::Hsla>,
    text_color: Option<gpui::Hsla>,
) -> InputStyle {
    let bg = if disabled {
        theme.surface.sunken
    } else {
        bg_color.unwrap_or_else(|| theme.surface.base)
    };

    let border = if disabled {
        theme.border.muted
    } else {
        border_color.unwrap_or_else(|| theme.border.default)
    };

    let focus_border = focus_border_color.unwrap_or_else(|| theme.border.focus);

    let text_color = if disabled {
        theme.content.disabled
    } else {
        text_color.unwrap_or_else(|| theme.content.primary)
    };

    InputStyle {
        bg,
        border,
        focus_border,
        text_color,
    }
}

/// Resolves the controlled/uncontrolled state for a component.
///
/// In "controlled" mode, the component's value is managed externally via the
/// `value` parameter and changes are communicated via `on_change`. In "uncontrolled"
/// mode, the component manages its own internal state.
///
/// # Parameters
/// - `external` - The externally provided value (controlled mode)
/// - `internal` - The internal state entity (uncontrolled mode)
/// - `cx` - The app context
/// - `default_value` - The default value to use if neither external nor internal is set
///
/// # Returns
/// The resolved value based on whether the component is controlled or uncontrolled.
pub fn resolve_controlled_state<T: Clone + Default + 'static>(
    external: Option<&T>,
    internal: Option<&Entity<T>>,
    cx: &App,
    default_value: T,
) -> T {
    if let Some(value) = external {
        return value.clone();
    }

    if let Some(internal) = internal {
        return internal.read(cx).clone();
    }

    default_value
}

/// Determines whether a component should use internal state management.
///
/// A component is "uncontrolled" (uses internal state) when:
/// - No external value is provided (`value` is None)
/// - No external change handler is provided (`on_change` is None)
///
/// # Parameters
/// - `has_value` - Whether an external value is provided
/// - `has_on_change` - Whether an on_change callback is provided
///
/// # Returns
/// `true` if the component should manage its own internal state.
pub fn use_internal_state(has_value: bool, has_on_change: bool) -> bool {
    !has_value && !has_on_change
}

/// Creates a keyed state for internal value management.
///
/// This is a convenience function that creates a use_keyed_state call
/// with a consistent prefix for input components.
///
/// # Parameters
/// - `window` - The window context
/// - `cx` - The app context
/// - `id` - The element ID for keying
/// - `key` - The state key string
/// - `default_value` - The default value for the state
///
/// # Returns
/// An optional Entity containing the internal state
pub fn create_internal_state<T: Clone + Default + 'static>(
    window: &mut Window,
    cx: &mut App,
    id: &ElementId,
    key: String,
    default_value: T,
    should_use: bool,
) -> Option<Entity<T>> {
    if should_use {
        Some(window.use_keyed_state((id.clone(), key), cx, |_, _| default_value))
    } else {
        None
    }
}

/// Updates the internal state value if it exists.
///
/// # Parameters
/// - `internal` - The internal state entity to update
/// - `cx` - The app context
/// - `new_value` - The new value to set
pub fn update_internal_state<T: Clone + 'static>(
    internal: &Option<Entity<T>>,
    cx: &mut App,
    new_value: T,
) {
    if let Some(internal) = internal {
        internal.update(cx, |state, _cx| {
            *state = new_value;
            _cx.notify();
        });
    }
}

/// Reads the value from internal state or returns the external value.
///
/// # Parameters
/// - `external` - The external value (if provided)
/// - `internal` - The internal state entity
/// - `cx` - The app context
///
/// # Returns
/// The resolved value
pub fn resolve_state_value<T: Clone + Default + 'static>(
    external: Option<&T>,
    internal: &Option<Entity<T>>,
    cx: &App,
) -> T {
    if let Some(external) = external {
        return external.clone();
    }

    if let Some(internal) = internal {
        return internal.read(cx).clone();
    }

    T::default()
}
