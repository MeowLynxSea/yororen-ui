//! `ComboBoxRenderer` — visual contract for `ComboBox`.
//!
//! Unlike `SelectRenderer` (which only paints a static trigger
//! + a fixed list of options), `ComboBoxRenderer::compose`
//!   receives `&mut Window` because the trigger embeds a real
//!   `TextInputElement` so the user can type to filter the
//!   option list. The text input's state is keyed by the
//!   combo box's id via `window.use_keyed_state`. Returning
//!   `AnyElement` matches `TextInputRenderer`'s shape (the
//!   `text_input` element is an `AnyElement` and can be
//!   embedded directly into the trigger area).
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / border / focus_border / fg / search_bg /
//! min_height / padding / border_radius) stay on the
//! concrete renderer type.

use std::any::Any;

use gpui::{AnyElement, App, Window};

use crate::headless::combo_box::ComboBoxProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct ComboBoxRenderState {
    pub open: bool,
    pub disabled: bool,
    pub has_value: bool,
    /// Editable mode (WinUI `IsEditable`). Renderers use this to
    /// swap the IBeam cursor / caret for a selection-only look.
    pub editable: bool,
    pub custom_bg: Option<gpui::Hsla>,
    pub custom_border: Option<gpui::Hsla>,
    pub custom_focus_border: Option<gpui::Hsla>,
    pub custom_fg: Option<gpui::Hsla>,
}

pub trait ComboBoxRenderer: Any + Send + Sync {
    fn compose(&self, props: &ComboBoxProps, cx: &mut App, window: &mut Window) -> AnyElement;
}
