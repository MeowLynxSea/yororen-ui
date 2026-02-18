//! Helper functions for UI components.
//!
//! This module provides common utility functions used across multiple components
//! to reduce code duplication.

use gpui::{App, ElementId, Entity, Window};

use crate::theme::{ActionVariantKind, Theme};

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
        bg_color.unwrap_or(theme.surface.base)
    };

    let border = if disabled {
        theme.border.muted
    } else {
        border_color.unwrap_or(theme.border.default)
    };

    let focus_border = focus_border_color.unwrap_or(theme.border.focus);

    let text_color = if disabled {
        theme.content.disabled
    } else {
        text_color.unwrap_or(theme.content.primary)
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

/// Determines whether a component should use internal state based on callback presence.
///
/// This is a simplified version for components that only need to check if a callback
/// is provided (e.g., checkbox, radio, switch, toggle_button).
/// A component is "uncontrolled" when no callback is provided.
///
/// # Parameters
/// - `has_on_change` - Whether an on_change callback is provided
///
/// # Returns
/// `true` if the component should manage its own internal state.
pub fn use_internal_state_simple(has_on_change: bool) -> bool {
    !has_on_change
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

/// Reads the value from internal state or returns the provided external value.
///
/// This is a version for components where the external value is always present
/// (not Option), like checkbox with `checked: bool`.
///
/// # Parameters
/// - `external` - The external value
/// - `internal` - The internal state entity
/// - `cx` - The app context
///
/// # Returns
/// The resolved value (internal if use_internal is true, otherwise external)
pub fn resolve_state_value_simple<T: Clone + 'static>(
    external: T,
    internal: &Option<Entity<T>>,
    cx: &App,
    use_internal: bool,
) -> T {
    if use_internal && let Some(internal) = internal {
        return internal.read(cx).clone();
    }
    external
}

/// Action component style configuration.
///
/// This struct holds the computed style values for action components
/// like Button, IconButton, ToggleButton, etc.
#[derive(Clone, Debug)]
pub struct ActionStyle {
    /// The background color.
    pub bg: gpui::Hsla,
    /// The hover background color.
    pub hover_bg: gpui::Hsla,
    /// The foreground/text color.
    pub fg: gpui::Hsla,
    /// The disabled background color.
    pub disabled_bg: gpui::Hsla,
    /// The disabled foreground/text color.
    pub disabled_fg: gpui::Hsla,
}

/// Computes the action style based on theme and component properties.
///
/// This function consolidates the common style resolution logic found in
/// Button, IconButton, and ToggleButton components.
///
/// # Parameters
/// - `theme` - The application theme
/// - `variant` - The action variant kind (Neutral, Primary, Danger)
/// - `disabled` - Whether the component is disabled
/// - `custom_bg` - Optional custom background color override
/// - `custom_hover_bg` - Optional custom hover background color override
///
/// # Returns
/// An `ActionStyle` struct containing the computed colors.
pub fn compute_action_style(
    theme: &Theme,
    variant: ActionVariantKind,
    disabled: bool,
    custom_bg: Option<gpui::Hsla>,
    custom_hover_bg: Option<gpui::Hsla>,
) -> ActionStyle {
    let action_variant = theme.action_variant(variant);

    if disabled {
        return ActionStyle {
            bg: action_variant.disabled_bg,
            hover_bg: action_variant.disabled_bg,
            fg: action_variant.disabled_fg,
            disabled_bg: action_variant.disabled_bg,
            disabled_fg: action_variant.disabled_fg,
        };
    }

    ActionStyle {
        bg: custom_bg.unwrap_or(action_variant.bg),
        hover_bg: custom_hover_bg.unwrap_or(action_variant.hover_bg),
        fg: action_variant.fg,
        disabled_bg: action_variant.disabled_bg,
        disabled_fg: action_variant.disabled_fg,
    }
}

/// Toggle component style configuration.
///
/// This struct holds the computed style values for toggle components
/// like Checkbox, Switch, Radio, etc.
#[derive(Clone, Debug)]
pub struct ToggleStyle {
    /// The background color when checked/selected.
    pub bg: gpui::Hsla,
    /// The border color when checked/selected.
    pub border: gpui::Hsla,
    /// The foreground/text/icon color when checked/selected.
    pub fg: gpui::Hsla,
    /// The background color on hover when checked/selected.
    pub hover_bg: gpui::Hsla,
    /// The opacity value when disabled.
    pub disabled_opacity: f32,
}

/// Computes the toggle style based on theme and component properties.
///
/// This function consolidates the common style resolution logic found in
/// Checkbox, Switch, Radio, and other toggle components.
///
/// # Parameters
/// - `theme` - The application theme
/// - `checked` - Whether the toggle is checked/selected
/// - `disabled` - Whether the component is disabled
/// - `custom_accent` - Optional custom accent color override
///
/// # Returns
/// A `ToggleStyle` struct containing the computed colors and disabled opacity.
pub fn compute_toggle_style(
    theme: &Theme,
    checked: bool,
    disabled: bool,
    custom_accent: Option<gpui::Hsla>,
) -> ToggleStyle {
    let accent = custom_accent.unwrap_or(theme.action.primary.bg);

    if disabled {
        return ToggleStyle {
            bg: theme.surface.sunken,
            border: theme.border.muted,
            fg: theme.content.disabled,
            hover_bg: theme.surface.sunken,
            disabled_opacity: 0.5,
        };
    }

    if checked {
        ToggleStyle {
            bg: accent,
            border: accent,
            fg: theme.action.primary.fg,
            hover_bg: theme.action.primary.hover_bg,
            disabled_opacity: 1.0,
        }
    } else {
        ToggleStyle {
            bg: theme.surface.base,
            border: theme.border.default,
            fg: theme.content.primary,
            hover_bg: theme.surface.hover,
            disabled_opacity: 1.0,
        }
    }
}
