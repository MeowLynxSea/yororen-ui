//! Text area actions module.
//!
//! Defines keyboard actions for text area editing.

use gpui::actions;

actions!(
    ui_text_area,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        Enter,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
    ]
);
