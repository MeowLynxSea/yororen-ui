//! Animation and timing constants for the UI library.

use std::time::Duration;

/// Cursor blink interval for text inputs.
pub const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// Animation durations for UI transitions.
pub mod animation {
    use super::Duration;

    /// Dropdown menu open/close animation.
    pub const MENU_OPEN: Duration = Duration::from_millis(160);

    /// Navigator slider animation.
    pub const NAVIGATOR_SLIDER: Duration = Duration::from_millis(200);

    /// Skeleton loading pulse animation (variant 1).
    pub const SKELETON_PULSE_1: Duration = Duration::from_millis(1100);

    /// Skeleton loading pulse animation (variant 2).
    pub const SKELETON_PULSE_2: Duration = Duration::from_millis(1200);

    /// Progress spinner animation.
    pub const PROGRESS_SPINNER: Duration = Duration::from_millis(850);

    /// Progress circle animation.
    pub const PROGRESS_CIRCLE: Duration = Duration::from_millis(900);
}
