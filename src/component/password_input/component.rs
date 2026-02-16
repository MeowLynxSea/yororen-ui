//! Password input component builder.

use std::sync::Arc;

use gpui::{
    div, App, CursorStyle, Div, ElementId,
    Hsla, InteractiveElement, MouseButton,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, prelude::FluentBuilder,
};

use super::state::{PasswordInputState, PasswordInputHandler};
use super::actions::*;
use super::element::PasswordLineElement;
use crate::action_handler;
use crate::theme::ActiveTheme;

#[derive(gpui::IntoElement)]
pub struct PasswordInput {
    element_id: ElementId,
    base: Div,
    placeholder: SharedString,

    disabled: bool,

    allow_copy: bool,
    allow_cut: bool,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<PasswordInputHandler>,
}

impl PasswordInput {
    pub fn new() -> Self {
        Self {
            element_id: "ui:password-input".into(),
            base: div().h(gpui::px(36.)).px_3(),
            placeholder: "".into(),

            disabled: false,

            allow_copy: false,
            allow_cut: false,

            bg: None,
            border: None,
            focus_border: None,
            text_color: None,
            height: None,
            on_change: None,
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

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Allow copy action to write selected text into clipboard.
    ///
    /// Default: `false`.
    pub fn allow_copy(mut self, allow: bool) -> Self {
        self.allow_copy = allow;
        self
    }

    /// Allow cut action to write selected text into clipboard and delete it.
    ///
    /// Default: `false`.
    pub fn allow_cut(mut self, allow: bool) -> Self {
        self.allow_cut = allow;
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border = Some(color.into());
        self
    }

    pub fn focus_border(mut self, color: impl Into<Hsla>) -> Self {
        self.focus_border = Some(color.into());
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
}

impl Default for PasswordInput {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for PasswordInput {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for PasswordInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl gpui::InteractiveElement for PasswordInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for PasswordInput {}

impl RenderOnce for PasswordInput {
    fn render(self, window: &mut gpui::Window, cx: &mut App) -> impl gpui::IntoElement {
        // PasswordInput requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id;

        let disabled = self.disabled;
        let allow_copy = self.allow_copy;
        let allow_cut = self.allow_cut;

        let state = window.use_keyed_state(id.clone(), cx, |_, cx| PasswordInputState::new(cx));
        let focus_handle = state.read(cx).focus_handle.clone();
        let placeholder = self.placeholder;
        state.update(cx, |state, _cx| {
            state.placeholder = placeholder;
        });

        let on_change = self.on_change;
        let last_content = window.use_keyed_state(
            (id.clone(), format!("{}:last-content", id)),
            cx,
            |_, _cx| SharedString::new_static(""),
        );

        let theme = cx.theme();

        let bg = if disabled {
            theme.surface.sunken
        } else {
            self.bg.unwrap_or_else(|| theme.surface.base)
        };

        let border_color = if disabled {
            theme.border.muted
        } else {
            self.border.unwrap_or_else(|| theme.border.default)
        };
        let focus_border_color = self
            .focus_border
            .unwrap_or_else(|| theme.border.focus);
        let text_color = if disabled {
            theme.content.disabled
        } else {
            self.text_color.unwrap_or_else(|| theme.content.primary)
        };
        let height = self.height.unwrap_or_else(|| gpui::px(36.).into());
        let inset = if disabled { gpui::px(6.) } else { gpui::px(5.) };

        let mut base = self
            .base
            .id(id.clone())
            .flex()
            .items_center()
            .w_full()
            .h(height)
            .rounded_md()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .when(!disabled && focus_handle.is_focused(window), |this| {
                this.border_2().border_color(focus_border_color)
            })
            .when(!disabled, |this| this.track_focus(&focus_handle))
            .when(!disabled, |this| this.cursor(CursorStyle::IBeam))
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.6))
            .key_context("UIPasswordInput")
            .on_action(action_handler!(state, disabled, Backspace, backspace))
            .on_action(action_handler!(state, disabled, Delete, delete))
            .on_action(action_handler!(state, disabled, Left, left))
            .on_action(action_handler!(state, disabled, Right, right))
            .on_action(action_handler!(state, disabled, SelectLeft, select_left))
            .on_action(action_handler!(state, disabled, SelectRight, select_right))
            .on_action(action_handler!(state, disabled, SelectAll, select_all))
            .on_action(action_handler!(state, disabled, Home, home))
            .on_action(action_handler!(state, disabled, End, end))
            .on_action(action_handler!(state, disabled, ShowCharacterPalette, show_character_palette))
            .on_action(action_handler!(state, disabled, Paste, paste))
            .on_action({
                let state = state.clone();
                move |action: &Cut, window, cx| {
                    if disabled || !allow_cut {
                        return;
                    }
                    state.update(cx, |state, cx| state.cut(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Copy, window, cx| {
                    if disabled || !allow_copy {
                        return;
                    }
                    state.update(cx, |state, cx| state.copy(action, window, cx));
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| {
                        state.focus_in(window, cx);
                        state.on_mouse_down(event, window, cx);
                    });
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let state = state.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.on_mouse_up(event, window, cx));
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let state = state.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.on_mouse_up(event, window, cx));
                }
            })
            .on_mouse_move({
                let state = state.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.on_mouse_move(event, window, cx));
                }
            });

        base = base
            .text_color(text_color)
            .child(
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .px(inset)
                    .child(div().w_full().rounded_sm().overflow_hidden().child(
                        PasswordLineElement {
                            input: state.clone(),
                            disabled,
                        },
                    )),
            )
            .on_mouse_down_out(move |_event, window, _cx| {
                if disabled {
                    return;
                }
                if focus_handle.is_focused(window) {
                    window.blur();
                }
            });

        base.map(move |this| {
            if on_change.is_none() {
                return this;
            }

            let on_change = on_change.expect("checked");
            let current = state.read(cx).content.clone();
            let prev = last_content.read(cx).clone();
            if current != prev {
                last_content.update(cx, |value, _cx| *value = current.clone());
                on_change(current, window, cx);
            }
            this
        })
    }
}
