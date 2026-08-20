//! Headless `checkbox` — checked / disabled state + on_toggle, no
//! visual.
//!
//! `apply` is purely a11y: focus + click. The caller (or the
//! renderer via `default_render`) owns every visual concern,
//! including hover / active feedback.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use super::switch::ToggleCallback;

#[derive(Clone)]
pub struct CheckboxProps {
    pub id: ElementId,
    pub checked: bool,
    /// Third ("mixed") state — the renderer paints a dash
    /// instead of a checkmark. Used by tri-state flows such as
    /// multi-select tree parents whose children are partially
    /// selected. Toggle semantics stay caller-owned: `on_toggle`
    /// still fires with `!checked`.
    pub indeterminate: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
    /// `true` if the caller supplied a custom checked-state color
    /// (consumed by `CheckboxRenderer.has_custom_tone`).
    pub has_custom_tone: bool,
    /// Caller-supplied override for the checked-state fill /
    /// border color. `None` → renderer falls back to the
    /// `action.primary` palette.
    pub custom_tone: Option<gpui::Hsla>,
}

pub fn checkbox(id: impl Into<ElementId>, cx: &mut App) -> CheckboxProps {
    CheckboxProps {
        id: id.into(),
        checked: false,
        indeterminate: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_toggle: None,
        has_custom_tone: false,
        custom_tone: None,
    }
}

impl CheckboxProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
    pub fn checked(mut self, v: bool) -> Self {
        self.checked = v;
        self
    }
    /// Set the third ("mixed") state. The renderer paints a
    /// dash; `on_toggle` still fires with `!checked` when the
    /// box is clicked, so tri-state flows can cycle
    /// indeterminate → checked → unchecked themselves.
    pub fn indeterminate(mut self, v: bool) -> Self {
        self.indeterminate = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn has_custom_tone(mut self, v: bool) -> Self {
        self.has_custom_tone = v;
        self
    }
    pub fn custom_tone(mut self, c: gpui::Hsla) -> Self {
        self.custom_tone = Some(c);
        self.has_custom_tone = true;
        self
    }
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(bool, Option<&ClickEvent>, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(f));
        self
    }

    /// Wire the headless contract onto the caller's `el`.
    ///
    /// Purely a11y: id, focus, click (which fires
    /// `on_toggle(!checked, ...)`). No visual feedback
    /// injected — caller / renderer owns hover / active.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            let checked = self.checked;
            s = s.on_click(move |ev, window, cx| {
                f(!checked, Some(ev), window, cx);
            });
        }
        s
    }

    /// Render the checkbox using the registered `CheckboxRenderer`.
    ///
    /// Data flow is one-way: the renderer takes the full
    /// `CheckboxProps` and returns a fully-built `Stateful<Div>`
    /// (visuals + check mark + hover / active + id + focus).
    /// Headless only layers the toggle callback on top.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::checkbox::CheckboxRenderer;
        use crate::renderer::markers::Checkbox as CheckboxMarker;

        let r: &Arc<dyn CheckboxRenderer> = cx
            .renderer_arc::<CheckboxMarker, dyn CheckboxRenderer>()
            .expect("CheckboxRenderer registered");

        let mut styled = r.compose(&self, &self.focus_handle, cx);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            let new_value = !self.checked;
            styled = styled.on_click(move |ev, window, cx| {
                f(new_value, Some(ev), window, cx);
            });
        }
        styled
    }
}
