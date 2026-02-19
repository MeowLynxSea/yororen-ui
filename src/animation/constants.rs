//! Animation and timing constants for the UI library.
//!
//! This module consolidates all animation duration constants used throughout
//! the library, providing a single source of truth for animation timing.

use std::time::Duration;

/// Cursor blink interval for text inputs.
pub const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// Animation durations for UI transitions.
pub mod duration {
    use super::Duration;

    // -------------------------------------------------------------------------
    // Menu / Dropdown animations
    // -------------------------------------------------------------------------

    /// Dropdown menu open/close animation.
    pub const MENU_OPEN: Duration = Duration::from_millis(160);

    /// Dropdown menu open/close animation (fast).
    pub const MENU_OPEN_FAST: Duration = Duration::from_millis(100);

    /// Dropdown menu open/close animation (slow).
    pub const MENU_OPEN_SLOW: Duration = Duration::from_millis(250);

    // -------------------------------------------------------------------------
    // Navigator / Slider animations
    // -------------------------------------------------------------------------

    /// Navigator slider animation.
    pub const NAVIGATOR_SLIDER: Duration = Duration::from_millis(200);

    /// Tab switch animation.
    pub const TAB_SWITCH: Duration = Duration::from_millis(150);

    // -------------------------------------------------------------------------
    // Loading / Skeleton animations
    // -------------------------------------------------------------------------

    /// Skeleton loading pulse animation (variant 1).
    pub const SKELETON_PULSE_1: Duration = Duration::from_millis(1100);

    /// Skeleton loading pulse animation (variant 2).
    pub const SKELETON_PULSE_2: Duration = Duration::from_millis(1200);

    // -------------------------------------------------------------------------
    // Progress / Spinner animations
    // -------------------------------------------------------------------------

    /// Progress spinner animation.
    pub const PROGRESS_SPINNER: Duration = Duration::from_millis(850);

    /// Progress circle animation.
    pub const PROGRESS_CIRCLE: Duration = Duration::from_millis(900);

    /// Progress bar indeterminate animation.
    pub const PROGRESS_BAR: Duration = Duration::from_millis(1500);

    // -------------------------------------------------------------------------
    // Modal / Dialog animations
    // -------------------------------------------------------------------------

    /// Modal fade in animation.
    pub const MODAL_FADE_IN: Duration = Duration::from_millis(200);

    /// Modal slide up animation.
    pub const MODAL_SLIDE_UP: Duration = Duration::from_millis(250);

    // -------------------------------------------------------------------------
    // Tooltip animations
    // -------------------------------------------------------------------------

    /// Tooltip show animation.
    pub const TOOLTIP_SHOW: Duration = Duration::from_millis(150);

    /// Tooltip hide animation.
    pub const TOOLTIP_HIDE: Duration = Duration::from_millis(100);

    // -------------------------------------------------------------------------
    // General purpose
    // -------------------------------------------------------------------------

    /// Very fast animation (for UI feedback).
    pub const VERY_FAST: Duration = Duration::from_millis(100);

    /// Fast animation (for simple transitions).
    pub const FAST: Duration = Duration::from_millis(150);

    /// Normal animation (default).
    pub const NORMAL: Duration = Duration::from_millis(200);

    /// Slow animation (for complex transitions).
    pub const SLOW: Duration = Duration::from_millis(300);

    /// Very slow animation (for emphasis).
    pub const VERY_SLOW: Duration = Duration::from_millis(400);

    /// Instant (0ms).
    pub const INSTANT: Duration = Duration::ZERO;
}
