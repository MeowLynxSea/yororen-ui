//! Password input component.
//!
//! A single-line password input component with masking.

mod actions;
mod state;
mod element;
mod component;

pub use actions::*;
pub use state::*;
pub use component::*;

use gpui::App;
use gpui::ElementId;

/// Creates a new password input element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn password_input(id: impl Into<ElementId>) -> PasswordInput {
    PasswordInput::new().id(id)
}

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        gpui::KeyBinding::new("backspace", Backspace, Some("UIPasswordInput")),
        gpui::KeyBinding::new("delete", Delete, Some("UIPasswordInput")),
        gpui::KeyBinding::new("left", Left, Some("UIPasswordInput")),
        gpui::KeyBinding::new("right", Right, Some("UIPasswordInput")),
        gpui::KeyBinding::new("shift-left", SelectLeft, Some("UIPasswordInput")),
        gpui::KeyBinding::new("shift-right", SelectRight, Some("UIPasswordInput")),
        gpui::KeyBinding::new("secondary-a", SelectAll, Some("UIPasswordInput")),
        gpui::KeyBinding::new("secondary-v", Paste, Some("UIPasswordInput")),
        gpui::KeyBinding::new("secondary-c", Copy, Some("UIPasswordInput")),
        gpui::KeyBinding::new("secondary-x", Cut, Some("UIPasswordInput")),
        gpui::KeyBinding::new("home", Home, Some("UIPasswordInput")),
        gpui::KeyBinding::new("end", End, Some("UIPasswordInput")),
        gpui::KeyBinding::new(
            "ctrl-secondary-space",
            ShowCharacterPalette,
            Some("UIPasswordInput"),
        ),
    ]);
}
