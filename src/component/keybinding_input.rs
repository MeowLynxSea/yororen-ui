use std::panic::Location;

use gpui::{
    Div, ElementId, Hsla, InteractiveElement, IntoElement, KeyDownEvent, Keystroke,
    ModifiersChangedEvent, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    component::{format_keybinding_ui, shortcut_hint},
    theme::ActiveTheme,
};

#[track_caller]
pub fn keybinding_input() -> KeybindingInput {
    KeybindingInput::new().id(ElementId::from(Location::caller()))
}

type ChangeFn = std::sync::Arc<dyn Fn(SharedString, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct KeybindingInput {
    element_id: Option<ElementId>,
    base: Div,

    value: Option<SharedString>,
    placeholder: SharedString,
    disabled: bool,

    bg_color: Option<Hsla>,
    border_color: Option<Hsla>,
    focus_border_color: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<ChangeFn>,
}

impl Default for KeybindingInput {
    fn default() -> Self {
        Self::new()
    }
}

impl KeybindingInput {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),
            value: None,
            placeholder: "Press keys…".into(),
            disabled: false,
            bg_color: None,
            border_color: None,
            focus_border_color: None,
            text_color: None,
            height: None,
            on_change: None,
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

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg_color = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn focus_border(mut self, color: impl Into<Hsla>) -> Self {
        self.focus_border_color = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    pub fn height(mut self, height: gpui::AbsoluteLength) -> Self {
        self.height = Some(height);
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(std::sync::Arc::new(handler));
        self
    }
}

impl ParentElement for KeybindingInput {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for KeybindingInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for KeybindingInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for KeybindingInput {}

impl RenderOnce for KeybindingInput {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let disabled = self.disabled;
        let theme = cx.theme().clone();

        let bg = if disabled {
            theme.surface.sunken
        } else {
            self.bg_color.unwrap_or(theme.surface.base)
        };

        let border_color = if disabled {
            theme.border.muted
        } else {
            self.border_color.unwrap_or(theme.border.default)
        };

        let focus_border_color = self.focus_border_color.unwrap_or(theme.border.focus);

        let text_color = if disabled {
            theme.content.disabled
        } else {
            self.text_color.unwrap_or(theme.content.primary)
        };

        let height = self.height.unwrap_or_else(|| px(36.).into());
        let placeholder = self.placeholder;

        let on_change = self.on_change;
        let use_internal_value = on_change.is_none();
        let initial_value = self
            .value
            .clone()
            .unwrap_or_else(|| SharedString::new_static(""));
        let internal_value = if use_internal_value {
            Some(
                window.use_keyed_state((id.clone(), "ui:keybinding:value"), cx, |_, _| {
                    initial_value
                }),
            )
        } else {
            None
        };

        let capture_active =
            window.use_keyed_state((id.clone(), "ui:keybinding:active"), cx, |_, _| false);

        let current_modifiers =
            window.use_keyed_state((id.clone(), "ui:keybinding:modifiers"), cx, |_, _| {
                gpui::Modifiers::default()
            });

        let focus_handle =
            window.use_keyed_state((id.clone(), "ui:keybinding:focus"), cx, |_, cx| {
                cx.focus_handle()
            });

        let on_change_state =
            window.use_keyed_state((id.clone(), "ui:keybinding:on-change"), cx, |_, _| {
                None::<ChangeFn>
            });
        on_change_state.update(cx, |state, _| *state = on_change.clone());

        let disabled_state =
            window.use_keyed_state((id.clone(), "ui:keybinding:disabled"), cx, |_, _| disabled);
        disabled_state.update(cx, |state, _| *state = disabled);

        let value = if use_internal_value {
            internal_value
                .as_ref()
                .expect("internal value should exist")
                .read(cx)
                .clone()
        } else {
            self.value
                .clone()
                .unwrap_or_else(|| SharedString::new_static(""))
        };

        let is_active = *capture_active.read(cx);
        let showing_placeholder = value.is_empty() && !is_active;

        // Install a keystroke interceptor once (per element ID) so we can capture chord/combination
        // keys before GPUI's pending-input mechanism delays dispatch.
        let _interceptor_subscription =
            window.use_keyed_state((id.clone(), "ui:keybinding:interceptor"), cx, {
                let capture_active = capture_active.clone();
                let focus_handle = focus_handle.clone();
                let current_modifiers = current_modifiers.clone();
                let internal_value = internal_value.clone();
                let on_change_state = on_change_state.clone();
                let disabled_state = disabled_state.clone();

                move |_, cx| {
                    Some(cx.intercept_keystrokes(move |event, window, cx| {
                        if *disabled_state.read(cx) {
                            return;
                        }
                        if !*capture_active.read(cx) {
                            return;
                        }
                        if !focus_handle.read(cx).is_focused(window) {
                            return;
                        }

                        // Keep the modifiers state fresh (for display) even when we stop propagation.
                        current_modifiers.update(cx, |state, _| *state = event.keystroke.modifiers);

                        let key = event.keystroke.key.to_ascii_lowercase();
                        let is_modifier_key = matches!(
                            key.as_str(),
                            "shift"
                                | "control"
                                | "ctrl"
                                | "alt"
                                | "option"
                                | "cmd"
                                | "meta"
                                | "super"
                                | "fn"
                        );
                        if event.keystroke.key.is_empty() || is_modifier_key {
                            return;
                        }

                        cx.stop_propagation();

                        let text = format_keybinding_ui(&event.keystroke);
                        if let Some(internal_value) = &internal_value {
                            internal_value.update(cx, |state, cx| {
                                *state = text.clone();
                                cx.notify();
                            });
                        }

                        if let Some(handler) = on_change_state.read(cx).as_ref().cloned() {
                            handler(text.clone(), window, cx);
                        }

                        capture_active.update(cx, |state, _| *state = false);
                        window.refresh();
                    }))
                }
            });

        let mods_for_display = *current_modifiers.read(cx);

        self.base
            .id(id.clone())
            .h(height)
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .rounded_md()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .focusable()
            .focus_visible(|style| style.border_2().border_color(focus_border_color))
            .track_focus(focus_handle.read(cx))
            .when(disabled, |this| this.opacity(0.6).cursor_not_allowed())
            .when(!disabled, |this| this.cursor_pointer())
            .on_click({
                let capture_active = capture_active.clone();
                let focus_handle = focus_handle.clone();
                move |_ev, window, cx| {
                    if disabled {
                        return;
                    }
                    let handle = focus_handle.read(cx).clone();
                    window.focus(&handle, cx);
                    capture_active.update(cx, |state, _| *state = true);

                    // Ensure we can actually receive key events.
                    window.refresh();
                }
            })
            .on_modifiers_changed({
                let capture_active = capture_active.clone();
                let current_modifiers = current_modifiers.clone();
                move |event: &ModifiersChangedEvent, _window, cx| {
                    if disabled || !*capture_active.read(cx) {
                        return;
                    }

                    current_modifiers.update(cx, |state, cx| {
                        *state = event.modifiers;
                        cx.notify();
                    });
                }
            })
            .capture_key_down({
                let capture_active = capture_active.clone();
                let current_modifiers = current_modifiers.clone();
                let internal_value = internal_value.clone();
                let on_change = on_change_state.clone();
                move |event: &KeyDownEvent, window, cx| {
                    if disabled || !*capture_active.read(cx) {
                        return;
                    }

                    // Stop propagation so text inputs / bindings don't consume the keystroke.
                    cx.stop_propagation();

                    let mut keystroke = event.keystroke.clone();
                    keystroke.modifiers = *current_modifiers.read(cx);
                    if keystroke.key.is_empty() {
                        return;
                    }

                    let text = format_keybinding_ui(&keystroke);
                    if let Some(internal_value) = &internal_value {
                        internal_value.update(cx, |state, cx| {
                            *state = text.clone();
                            cx.notify();
                        });
                    }

                    if let Some(handler) = on_change.read(cx).as_ref().cloned() {
                        handler(text.clone(), window, cx);
                    }

                    capture_active.update(cx, |state, _| *state = false);
                }
            })
            .child(
                div().flex().items_center().gap_2().child(
                    (if is_active {
                        let ks = Keystroke {
                            modifiers: mods_for_display,
                            ..Default::default()
                        };
                        let hint = if ks.modifiers.modified() {
                            format_keybinding_ui(&ks)
                        } else {
                            SharedString::new_static("等待按键…")
                        };

                        div()
                            .text_color(theme.content.secondary)
                            .child(hint)
                            .into_any_element()
                    } else if showing_placeholder {
                        div()
                            .text_color(theme.content.tertiary)
                            .child(placeholder)
                            .into_any_element()
                    } else {
                        div()
                            .font_family("monospace")
                            .text_color(text_color)
                            .child(value)
                            .into_any_element()
                    })
                    .into_any_element(),
                ),
            )
            .child(shortcut_hint("Press keys"))
    }
}
