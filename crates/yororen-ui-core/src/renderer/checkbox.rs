//! `CheckboxRenderer` — visual contract for `Checkbox`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{App, Div, FocusHandle, Hsla, Stateful};

use crate::headless::checkbox::CheckboxProps;

/// Projection of `CheckboxProps` used by built-in renderers
/// when they want to factor out helpers. Not part of the
/// `CheckboxRenderer` trait surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct CheckboxRenderState {
    pub checked: bool,
    /// Third ("mixed") state — paints a dash instead of a
    /// checkmark. Used by tri-state flows such as multi-select
    /// tree parents whose children are partially selected.
    /// Renderers treat `checked || indeterminate` as the "on"
    /// state for fill / border colours.
    pub indeterminate: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
    pub custom_tone: Option<Hsla>,
}

pub trait CheckboxRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for a checkbox.
    fn compose(&self, props: &CheckboxProps, focus_handle: &FocusHandle, cx: &App)
    -> Stateful<Div>;
}
