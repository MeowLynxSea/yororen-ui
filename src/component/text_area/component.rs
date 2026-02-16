//! Text area component builder.

use std::sync::Arc;

use gpui::{
    div, App, CursorStyle, Div, ElementId,
    Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, prelude::FluentBuilder,
};

use super::state::{TextAreaState, WrapMode, EnterBehavior, TextAreaHandler};
use super::actions::*;
use super::element::TextAreaElement;
use crate::action_handler;
use crate::theme::ActiveTheme;

#[derive(IntoElement)]
pub struct TextArea {
    element_id: ElementId,
    base: Div,
    placeholder: SharedString,

    disabled: bool,
    wrap: WrapMode,
    enter: EnterBehavior,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<TextAreaHandler>,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            element_id: "ui:text-area".into(),
            base: div().h(gpui::px(120.)).px_3(),
            placeholder: "".into(),

            disabled: false,
            wrap: WrapMode::None,
            enter: EnterBehavior::Newline,

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

    pub fn wrap(mut self, wrap: WrapMode) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn enter_behavior(mut self, enter: EnterBehavior) -> Self {
        self.enter = enter;
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

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TextArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for TextArea {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for TextArea {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for TextArea {}

impl RenderOnce for TextArea {
    fn render(self, window: &mut gpui::Window, cx: &mut App) -> impl IntoElement {
        let id = self.element_id;

        let disabled = self.disabled;
        let state = window.use_keyed_state(id.clone(), cx, |_, cx| TextAreaState::new(cx));
        let focus_handle = state.read(cx).focus_handle.clone();

        let placeholder = self.placeholder;
        let wrap = self.wrap;
        let enter = self.enter;
        state.update(cx, |state, _cx| {
            state.placeholder = placeholder;
            state.wrap = wrap;
            state.enter = enter;
        });

        let on_change = self.on_change;
        let last_content =
            window.use_keyed_state((id.clone(), format!("{}:last-content", id)), cx, |_, _cx| {
                SharedString::new_static("")
            });

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
        let height = self.height.unwrap_or_else(|| gpui::px(120.).into());
        let inset = if disabled { gpui::px(6.) } else { gpui::px(5.) };

        let mut base = self
            .base
            .id(id.clone())
            .flex()
            .items_start()
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
            .key_context("UITextArea")
            .on_action(action_handler!(state, disabled, Backspace, backspace))
            .on_action(action_handler!(state, disabled, Delete, delete))
            .on_action(action_handler!(state, disabled, Left, left))
            .on_action(action_handler!(state, disabled, Right, right))
            .on_action(action_handler!(state, disabled, Up, up))
            .on_action(action_handler!(state, disabled, Down, down))
            .on_action(action_handler!(state, disabled, SelectLeft, select_left))
            .on_action(action_handler!(state, disabled, SelectRight, select_right))
            .on_action(action_handler!(state, disabled, SelectUp, select_up))
            .on_action(action_handler!(state, disabled, SelectDown, select_down))
            .on_action(action_handler!(state, disabled, SelectAll, select_all))
            .on_action(action_handler!(state, disabled, Home, home))
            .on_action(action_handler!(state, disabled, End, end))
            .on_action(action_handler!(state, disabled, Enter, enter))
            .on_action(action_handler!(state, disabled, ShowCharacterPalette, show_character_palette))
            .on_action(action_handler!(state, disabled, Paste, paste))
            .on_action(action_handler!(state, disabled, Cut, cut))
            .on_action({
                let state = state.clone();
                move |action: &Copy, window, cx| {
                    if disabled {
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
                div().w_full().h_full().flex().px(inset).child(
                    div()
                        .w_full()
                        .h_full()
                        .rounded_sm()
                        .id(format!("{}:scroll", id))
                        .overflow_scroll()
                        .child(TextAreaElement {
                            input: state.clone(),
                            disabled,
                        }),
                ),
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
            let current = state.read(cx).edit.content().clone();
            let prev = last_content.read(cx).clone();
            if current != prev {
                last_content.update(cx, |value, _cx| *value = current.clone());
                on_change(current, window, cx);
            }
            this
        })
    }
}
