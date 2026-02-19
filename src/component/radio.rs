use std::sync::Arc;

use gpui::{
    Animation, AnimationExt, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    animation,
    component::{
        ToggleCallback, compute_toggle_style, create_internal_state, resolve_state_value_simple,
        use_internal_state_simple,
    },
    theme::ActiveTheme,
};

use crate::animation::ease_in_out_clamped;

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
pub fn radio(id: impl Into<ElementId>) -> Radio {
    Radio::new().id(id)
}

#[derive(IntoElement)]
pub struct Radio {
    element_id: ElementId,
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
            element_id: "ui:radio".into(),
            base: div().w(px(18.)).h(px(18.)),
            checked: false,
            disabled: false,
            on_toggle: None,
            tone: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
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

        // Radio requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id;

        let use_internal = use_internal_state_simple(on_toggle.is_some());
        let internal_checked = create_internal_state(
            window,
            cx,
            &id,
            "ui:radio:checked".to_string(),
            explicit_checked,
            use_internal,
        );

        let checked =
            resolve_state_value_simple(explicit_checked, &internal_checked, cx, use_internal);

        let theme = cx.theme();
        let toggle_style = compute_toggle_style(theme, checked, disabled, tone);

        let mut base = self
            .base
            .id(id.clone())
            .rounded_full()
            .border_1()
            .border_color(toggle_style.border)
            .bg(toggle_style.bg)
            .flex()
            .items_center()
            .justify_center()
            .focusable()
            .focus_visible(|style| style.border_2().border_color(theme.border.focus));

        if disabled {
            base = base
                .opacity(toggle_style.disabled_opacity)
                .cursor_not_allowed();
        } else {
            base = base
                .cursor_pointer()
                .hover(move |this| this.bg(toggle_style.hover_bg));
        }

        // Add animated inner dot for checked state
        let inner_dot = div().w(px(8.)).h(px(8.)).rounded_full().bg(toggle_style.fg);

        let animated_dot = inner_dot.with_animation(
            format!("ui:radio:dot:{}", checked),
            Animation::new(animation::duration::FAST).with_easing(ease_in_out_clamped),
            move |this, value| this.opacity(if checked { value } else { 1.0 - value }),
        );

        base = base.when(checked, |this| {
            this.border_color(toggle_style.border)
                .text_color(toggle_style.fg)
                .child(animated_dot)
        });

        base.on_click(move |ev, window, cx| {
            if disabled {
                return;
            }

            if use_internal {
                if let Some(internal_checked) = &internal_checked {
                    internal_checked.update(cx, |value, _cx| *value = !*value);
                }
            } else if let Some(handler) = &on_toggle {
                handler(!explicit_checked, Some(ev), window, cx);
            }
        })
    }
}
