use gpui::{
    ClickEvent, Div, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{component::generate_element_id, theme::{ActionVariantKind, ActiveTheme}};

/// Creates a new toggle button element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn toggle_button(label: impl Into<String>) -> ToggleButton {
    ToggleButton::new(label)
}

type ToggleFn = Box<dyn Fn(bool, &ClickEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct ToggleButton {
    element_id: Option<ElementId>,
    base: Div,
    label: String,
    selected: bool,
    disabled: bool,
    on_toggle: Option<ToggleFn>,
    variant: ActionVariantKind,
    default_selected: bool,
    bg: Option<Hsla>,
    selected_bg: Option<Hsla>,
    hover_bg: Option<Hsla>,

    group: Option<String>,
}

impl ToggleButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            element_id: None,
            base: div().h(px(36.)).px_4().py_2(),
            label: label.into(),
            selected: false,
            disabled: false,
            on_toggle: None,
            variant: ActionVariantKind::Neutral,
            default_selected: false,
            bg: None,
            selected_bg: None,
            hover_bg: None,
            group: None,
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

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ActionVariantKind) -> Self {
        self.variant = variant;
        self
    }

    pub fn default_selected(mut self, default_selected: bool) -> Self {
        self.default_selected = default_selected;
        self
    }

    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(bool, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.bg = Some(fill.into());
        self
    }

    pub fn selected_bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.selected_bg = Some(fill.into());
        self
    }

    pub fn hover_bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.hover_bg = Some(fill.into());
        self
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

impl ParentElement for ToggleButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ToggleButton {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for ToggleButton {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for ToggleButton {}

impl RenderOnce for ToggleButton {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let selected = self.selected;
        let disabled = self.disabled;
        let on_toggle = self.on_toggle;
        let bg = self.bg;
        let selected_bg = self.selected_bg;
        let hover_bg = self.hover_bg;
        let variant = self.variant;
        let group = self.group;
        let default_selected = self.default_selected;
        let element_id = self.element_id;

        // ToggleButton requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = element_id.unwrap_or_else(|| generate_element_id("ui:toggle-button"));

        let use_internal_state = on_toggle.is_none();
        let internal_selected = use_internal_state
            .then(|| window.use_keyed_state(id.clone(), cx, |_window, _cx| selected));

        let group_selected = group.as_ref().map(|group| {
            let group_id = format!("toggle-group:{}", group);
            window.use_keyed_state(group_id, cx, |_window, _cx| None::<ElementId>)
        });

        let group_explicit = group.as_ref().map(|group| {
            let group_id = format!("toggle-group-explicit:{}", group);
            window.use_keyed_state(group_id, cx, |_window, _cx| false)
        });

        if let (Some(group_selected), Some(group_explicit)) = (&group_selected, &group_explicit) {
            let is_explicit = *group_explicit.read(cx);
            let current_selected = group_selected.read(cx).clone();
            if !is_explicit {
                if default_selected {
                    group_selected.update(cx, |value, _cx| *value = Some(id.clone()));
                } else if current_selected.is_none() {
                    group_selected.update(cx, |value, _cx| *value = Some(id.clone()));
                }
            }
        }

        let resolved_selected = if use_internal_state {
            if let Some(group_selected) = &group_selected {
                group_selected.read(cx).as_ref() == Some(&id)
            } else {
                *internal_selected
                    .as_ref()
                    .expect("internal state should exist")
                    .read(cx)
            }
        } else {
            selected
        };

        let mut base = self
            .base
            .id(id.clone())
            .rounded_md()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .focusable();

        if disabled {
            base = base.opacity(0.5).cursor_not_allowed();
        }

        let action_variant = cx.theme().action_variant(variant);
        let selected_variant = &cx.theme().action.primary;

        let mut resolved_bg = if resolved_selected {
            selected_bg.unwrap_or(selected_variant.bg)
        } else {
            bg.unwrap_or(action_variant.bg)
        };

        let mut resolved_hover_bg = if resolved_selected {
            selected_variant.hover_bg
        } else {
            hover_bg.unwrap_or(action_variant.hover_bg)
        };

        let mut resolved_text_color = if resolved_selected {
            selected_variant.fg
        } else {
            action_variant.fg
        };

        if disabled {
            resolved_bg = if resolved_selected {
                selected_variant.disabled_bg
            } else {
                action_variant.disabled_bg
            };
            resolved_hover_bg = resolved_bg;
            resolved_text_color = if resolved_selected {
                selected_variant.disabled_fg
            } else {
                action_variant.disabled_fg
            };
        }

        base = base
            .bg(resolved_bg)
            .when(resolved_selected, |this| {
                this.border_1().border_color(cx.theme().border.default)
            })
            .text_color(resolved_text_color)
            .hover(move |this| this.bg(resolved_hover_bg))
            .focus_visible(|style| style.border_2().border_color(cx.theme().border.focus))
            .child(self.label);

        base.on_click(move |ev, window, cx| {
            if disabled {
                return;
            }

            if use_internal_state {
                if let Some(group_selected) = &group_selected {
                    group_selected.update(cx, |value, _cx| {
                        if value.as_ref() != Some(&id) {
                            *value = Some(id.clone());
                        }
                    });
                    if let Some(group_explicit) = &group_explicit {
                        group_explicit.update(cx, |value, _cx| *value = true);
                    }
                } else if let Some(internal_selected) = &internal_selected {
                    internal_selected.update(cx, |value, _cx| *value = !*value);
                }
            } else if let Some(handler) = &on_toggle {
                handler(!selected, ev, window, cx);
            }
        })
    }
}
