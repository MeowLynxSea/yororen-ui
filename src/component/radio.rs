use std::sync::Arc;

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    component::{generate_element_id, ToggleCallback},
    theme::ActiveTheme,
};

/// Creates a new radio button element.
/// Requires an id to be set via `.id()` for internal state management.
///
/// # Accessibility
///
/// This component provides accessibility support:
/// - The radio button is keyboard accessible (Tab to focus, Space/Enter to select)
/// - The checked state is visually indicated with a filled circle
/// - Disabled state is properly conveyed to assistive technologies
///
/// For full accessibility support:
/// - Use with a `<label>` element for proper text association
/// - Group related radio buttons using `RadioGroup` for proper mutual exclusion
/// - The component internally manages `role="radio"` and `aria-checked` state
pub fn radio() -> Radio {
    Radio::new()
}

#[derive(IntoElement)]
pub struct Radio {
    element_id: Option<ElementId>,
    base: Div,
    checked: bool,
    disabled: bool,
    on_toggle: Option<ToggleCallback>,
    tone: Option<Hsla>,
}

impl Default for Radio {
    fn default() -> Self {
        Self::new()
    }
}

impl Radio {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div().w(px(18.)).h(px(18.)),
            checked: false,
            disabled: false,
            on_toggle: None,
            tone: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tone(mut self, tone: impl Into<Hsla>) -> Self {
        self.tone = Some(tone.into());
        self
    }

    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(bool, Option<&ClickEvent>, &mut gpui::Window, &mut gpui::App),
    {
        self.on_toggle = Some(Arc::new(handler));
        self
    }
}

impl ParentElement for Radio {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Radio {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Radio {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Radio {}

impl RenderOnce for Radio {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let explicit_checked = self.checked;
        let on_toggle = self.on_toggle;
        let tone = self.tone;
        let element_id = self.element_id;

        // Radio requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = element_id.unwrap_or_else(|| generate_element_id("ui:radio"));

        let use_internal_state = on_toggle.is_none();
        let internal_checked = use_internal_state
            .then(|| window.use_keyed_state(id.clone(), cx, |_window, _cx| explicit_checked));

        let checked = if use_internal_state {
            *internal_checked
                .as_ref()
                .expect("internal state should exist")
                .read(cx)
        } else {
            explicit_checked
        };

        let theme = cx.theme();
        let accent = tone.unwrap_or_else(|| theme.action.primary.bg);

        let bg = if checked { accent } else { theme.surface.base };

        let border = if checked {
            accent
        } else {
            theme.border.default
        };

        let fg = if checked {
            theme.action.primary.fg
        } else {
            theme.content.primary
        };

        let hover_bg = theme.surface.hover;

        let mut base = self
            .base
            .id(id.clone())
            .rounded_full()
            .border_1()
            .border_color(border)
            .bg(bg)
            .flex()
            .items_center()
            .justify_center()
            .focusable()
            .focus_visible(|style| style.border_2().border_color(theme.border.focus));

        if disabled {
            base = base.opacity(0.5).cursor_not_allowed();
        } else {
            base = base.cursor_pointer().hover(move |this| this.bg(hover_bg));
        }

        base = base.when(checked, |this| this.border_color(accent).text_color(fg));

        base.on_click(move |ev, window, cx| {
            if disabled {
                return;
            }

            if use_internal_state {
                if let Some(internal_checked) = &internal_checked {
                    internal_checked.update(cx, |value, _cx| *value = !*value);
                }
            } else if let Some(handler) = &on_toggle {
                handler(!explicit_checked, Some(ev), window, cx);
            }
        })
    }
}
