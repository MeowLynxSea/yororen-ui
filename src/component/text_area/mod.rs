//! Text area component.
//!
//! A multi-line text input component with editing capabilities.

mod actions;
mod component;
mod element;
mod layout;
mod state;

pub use actions::*;
pub use component::*;
pub use state::*;

use gpui::{App, ElementId};

/// Creates a new text area element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn text_area(id: impl Into<ElementId>) -> TextArea {
    TextArea::new().id(id)
}

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        gpui::KeyBinding::new("backspace", Backspace, Some("UITextArea")),
        gpui::KeyBinding::new("delete", Delete, Some("UITextArea")),
        gpui::KeyBinding::new("left", Left, Some("UITextArea")),
        gpui::KeyBinding::new("right", Right, Some("UITextArea")),
        gpui::KeyBinding::new("up", Up, Some("UITextArea")),
        gpui::KeyBinding::new("down", Down, Some("UITextArea")),
        gpui::KeyBinding::new("shift-left", SelectLeft, Some("UITextArea")),
        gpui::KeyBinding::new("shift-right", SelectRight, Some("UITextArea")),
        gpui::KeyBinding::new("shift-up", SelectUp, Some("UITextArea")),
        gpui::KeyBinding::new("shift-down", SelectDown, Some("UITextArea")),
        gpui::KeyBinding::new("secondary-a", SelectAll, Some("UITextArea")),
        gpui::KeyBinding::new("secondary-v", Paste, Some("UITextArea")),
        gpui::KeyBinding::new("secondary-c", Copy, Some("UITextArea")),
        gpui::KeyBinding::new("secondary-x", Cut, Some("UITextArea")),
        gpui::KeyBinding::new("home", Home, Some("UITextArea")),
        gpui::KeyBinding::new("end", End, Some("UITextArea")),
        gpui::KeyBinding::new("enter", Enter, Some("UITextArea")),
        gpui::KeyBinding::new(
            "ctrl-secondary-space",
            ShowCharacterPalette,
            Some("UITextArea"),
        ),
    ]);
}
