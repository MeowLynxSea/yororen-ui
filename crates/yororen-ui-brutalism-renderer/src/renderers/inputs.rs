//! Brutalist input renderers: `TextInput`, `TextArea`,
//! `PasswordInput`, `NumberInput`, `FilePathInput`,
//! `SearchInput`, `Select`, `ComboBox`, `KeybindingInput`.
//!
//! Each renderer's `compose` owns the full visual pipeline.

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, CursorStyle, Div, ElementId, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement, Pixels,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, hsla,
    prelude::FluentBuilder, px,
};
use yororen_ui_core::action_handler;
use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::file_path_input::FilePathInputProps;
use yororen_ui_core::headless::icon::{IconSource, icon};
use yororen_ui_core::headless::keybinding_input::{KeybindingInputMode, KeybindingInputProps};
use yororen_ui_core::headless::number_input::NumberInputProps;
use yororen_ui_core::headless::password_input::PasswordInputProps;
use yororen_ui_core::headless::search_input::SearchInputProps;
use yororen_ui_core::headless::text_area::TextAreaProps;
use yororen_ui_core::headless::text_area_element::TextAreaElement;
use yororen_ui_core::headless::text_input::{
    Backspace, Copy, Cut, Delete, End, Enter, Escape, Home, Left, Paste, Right, SelectAll,
    SelectLeft, SelectRight, ShowCharacterPalette, TextInputProps, TextInputState,
};
use yororen_ui_core::headless::text_input_element::{
    TextInputElement, start_cursor_blink, wire_input_keyboard,
};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;
use yororen_ui_default_renderer::animation::AnimatedPresenceElement;

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS, brutal_border_color};

// =====================================================================
// Shared brutalist input colours.
// =====================================================================

fn brutal_input_bg(disabled: bool, theme: &Theme) -> Hsla {
    if disabled {
        theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
    } else {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
}

fn brutal_input_border(disabled: bool, theme: &Theme) -> Hsla {
    if disabled {
        theme
            .get_color("border.muted")
            .unwrap_or(brutal_border_color(theme))
    } else {
        brutal_border_color(theme)
    }
}

fn brutal_input_focus_border(theme: &Theme) -> Hsla {
    theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
}

fn brutal_input_text_color(disabled: bool, theme: &Theme) -> Hsla {
    if disabled {
        theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
    } else {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
}

fn brutal_input_hint_color(theme: &Theme) -> Hsla {
    theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
}

fn brutal_input_min_height(theme: &Theme) -> Pixels {
    px(theme
        .get_number("tokens.control.input.min_height")
        .unwrap_or(42.0) as f32)
}

fn brutal_input_padding(theme: &Theme) -> Edges<Pixels> {
    let h = theme
        .get_number("tokens.control.input.horizontal_padding")
        .unwrap_or(12.0) as f32;
    let v = theme
        .get_number("tokens.control.input.vertical_padding")
        .unwrap_or(10.0) as f32;
    Edges::symmetric(px(h), px(v))
}

// =====================================================================
// TextInput
// =====================================================================

pub use yororen_ui_core::renderer::text_input::{TextInputRenderState, TextInputRenderer};

pub struct BrutalTextInputRenderer;

impl TextInputRenderer for BrutalTextInputRenderer {
    fn compose(&self, props: &TextInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        let placeholder_str = props.placeholder.clone();
        let max_length = props.max_length;
        let disabled = props.disabled;
        let on_change = props.on_change.clone();
        let on_submit = props.on_submit.clone();
        let initial_value = props.initial_value.clone();

        let state = window.use_keyed_state(props.id.clone(), cx, |_window, cx| {
            let mut s = TextInputState::new(&mut *cx);
            if let Some(v) = initial_value.clone() {
                s.value = v;
            }
            s
        });
        state.update(cx, |s, _cx| {
            s.placeholder = SharedString::from(placeholder_str);
            s.max_length = max_length;
            s.on_change = on_change;
            s.on_submit = on_submit.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);
        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }
        let placeholder_for_element = state.read(cx).placeholder.clone();

        let theme = cx.theme().clone();
        let bg = brutal_input_bg(disabled, &theme);
        let border_color = if focused {
            brutal_input_focus_border(&theme)
        } else {
            brutal_input_border(disabled, &theme)
        };
        let text_color = brutal_input_text_color(disabled, &theme);
        let hint_color = brutal_input_hint_color(&theme);
        // The cursor is normally the focus border (a vivid
        // accent colour). But when the caller suppresses the
        // focus border (e.g. the combo_box embedding this
        // text input in its trigger), the focus border colour
        // is transparent — the cursor would vanish. Fall back
        // to the text colour so the caret is always visible.
        let cursor_color = if props.has_custom_focus_border {
            text_color
        } else {
            brutal_input_focus_border(&theme)
        };
        let selection_color = {
            let c = if props.has_custom_focus_border {
                text_color
            } else {
                brutal_input_focus_border(&theme)
            };
            hsla(c.h, c.s, c.l, 0.5)
        };
        let min_h = brutal_input_min_height(&theme);
        let padding = brutal_input_padding(&theme);
        // When `has_custom_border` is set (e.g. the combo_box
        // embedding this text input in its trigger) we want
        // the hover state to keep the same custom border, not
        // snap back to the theme's `border.default`. The
        // default brutalist hover is
        // `.hover(|s| s.border_color(brutal_border_color(&theme)))`
        // which would re-introduce the very ring the caller
        // tried to suppress.
        let hover_border = if props.has_custom_border {
            border_color
        } else {
            brutal_border_color(&theme)
        };
        let active_border = brutal_input_focus_border(&theme);
        drop(theme);

        let opacity = if disabled { 0.6 } else { 1.0 };

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color,
            cursor_color,
            selection_color,
            placeholder: placeholder_for_element,
            value_override: None,
        };

        let styled: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_3()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(px(BRUTAL_RADIUS))
            .opacity(opacity)
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .track_focus(&focus_handle)
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(inner);

        let keyed = wire_input_keyboard(
            styled,
            state.clone(),
            focus_handle.clone(),
            disabled,
            on_submit,
        );
        keyed.into_any_element()
    }
}

// =====================================================================
// TextArea
// =====================================================================

pub use yororen_ui_core::renderer::text_area::{TextAreaRenderState, TextAreaRenderer};

pub struct BrutalTextAreaRenderer;

impl TextAreaRenderer for BrutalTextAreaRenderer {
    fn compose(&self, props: &TextAreaProps, cx: &mut App, window: &mut Window) -> AnyElement {
        let placeholder_str = props.placeholder.clone();
        let disabled = props.disabled;
        let max_length = props.max_length;
        let on_change = props.on_change.clone();

        let state = window.use_keyed_state(props.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = SharedString::from(placeholder_str);
            s.max_length = max_length;
            s.on_change = on_change.clone();
            s.paste_newlines = true;
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);
        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let theme = cx.theme().clone();
        let bg = brutal_input_bg(disabled, &theme);
        let border_color = if focused {
            brutal_input_focus_border(&theme)
        } else {
            brutal_input_border(disabled, &theme)
        };
        let text_color = brutal_input_text_color(disabled, &theme);
        let hint_color = brutal_input_hint_color(&theme);
        // See text_input: with `has_custom_focus_border: true`
        // the focus border (and therefore the cursor) is
        // transparent. Fall back to the text colour so the
        // caret stays visible.
        let cursor_color = if props.has_custom_focus_border {
            text_color
        } else {
            brutal_input_focus_border(&theme)
        };
        let selection_color = {
            let c = if props.has_custom_focus_border {
                text_color
            } else {
                brutal_input_focus_border(&theme)
            };
            hsla(c.h, c.s, c.l, 0.5)
        };
        let min_h = px(theme
            .get_number("tokens.control.text_area.min_height")
            .unwrap_or(90.0) as f32);
        let pad = px(theme
            .get_number("tokens.control.text_area.padding")
            .unwrap_or(12.0) as f32);
        let hover_border = brutal_border_color(&theme);
        let active_border = brutal_input_focus_border(&theme);
        drop(theme);

        let placeholder_for_element = state.read(cx).placeholder.clone();
        let inner = TextAreaElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color,
            cursor_color,
            selection_color,
            placeholder: placeholder_for_element,
            min_h,
        };

        let base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_3()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(px(BRUTAL_RADIUS))
            .p(pad)
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .child(inner)
            .track_focus(&focus_handle);

        let mut keyed: Stateful<Div> = base
            .key_context("UITextInput")
            .on_action(action_handler!(
                state.clone(),
                disabled,
                Backspace,
                backspace
            ))
            .on_action(action_handler!(state.clone(), disabled, Delete, delete))
            .on_action(action_handler!(state.clone(), disabled, Left, left))
            .on_action(action_handler!(state.clone(), disabled, Right, right))
            .on_action(action_handler!(
                state.clone(),
                disabled,
                SelectLeft,
                select_left
            ))
            .on_action(action_handler!(
                state.clone(),
                disabled,
                SelectRight,
                select_right
            ))
            .on_action(action_handler!(
                state.clone(),
                disabled,
                SelectAll,
                select_all
            ))
            .on_action(action_handler!(state.clone(), disabled, Home, home))
            .on_action(action_handler!(state.clone(), disabled, End, end))
            .on_action(action_handler!(
                state.clone(),
                disabled,
                ShowCharacterPalette,
                show_character_palette
            ))
            .on_action(action_handler!(state.clone(), disabled, Paste, paste))
            .on_action(action_handler!(state.clone(), disabled, Cut, cut))
            .on_action(action_handler!(state.clone(), disabled, Copy, copy));

        let state_for_enter = state.clone();
        let on_change_for_enter = on_change.clone();
        keyed = keyed.on_action(move |_: &Enter, window, cx| {
            if disabled {
                return;
            }
            let before = state_for_enter.read(cx).value.clone();
            state_for_enter.update(cx, |s, _cx| {
                s.insert_text("\n");
            });
            let after = state_for_enter.read(cx).value.clone();
            if before != after
                && let Some(cb) = on_change_for_enter.as_ref()
            {
                cb(&after, window, &mut *cx);
            }
        });

        let state_for_mouse = state.clone();
        let state_for_move = state.clone();
        let state_for_up = state.clone();
        keyed = keyed
            .on_mouse_down(
                MouseButton::Left,
                move |event: &MouseDownEvent, window, cx| {
                    state_for_mouse.update(cx, |s, cx| {
                        s.on_mouse_down(event.position, window, cx);
                    });
                },
            )
            .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                state_for_up.update(cx, |s, cx| s.on_mouse_up(event, window, cx));
            })
            .on_mouse_move(move |event: &MouseMoveEvent, window, cx| {
                state_for_move.update(cx, |s, cx| s.on_mouse_move(event, window, cx));
            });

        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .into_any_element()
    }
}

// =====================================================================
// PasswordInput
// =====================================================================

pub use yororen_ui_core::renderer::password_input::{
    PasswordInputRenderState, PasswordInputRenderer,
};

pub struct BrutalPasswordInputRenderer;

impl PasswordInputRenderer for BrutalPasswordInputRenderer {
    fn compose(&self, props: &PasswordInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        let placeholder_str = props.placeholder.clone();
        let disabled = props.disabled;
        let max_length = props.max_length;
        let on_change = props.on_change.clone();
        let on_submit = props.on_submit.clone();
        let mask_char = props.mask_char;

        let state = window.use_keyed_state(props.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = SharedString::from(placeholder_str);
            s.max_length = max_length;
            s.on_change = on_change.clone();
            s.on_submit = on_submit.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);
        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let theme = cx.theme().clone();
        let bg = brutal_input_bg(disabled, &theme);
        let border_color = if focused {
            brutal_input_focus_border(&theme)
        } else {
            brutal_input_border(disabled, &theme)
        };
        let text_color = brutal_input_text_color(disabled, &theme);
        let hint_color = brutal_input_hint_color(&theme);
        let min_h = brutal_input_min_height(&theme);
        let padding = brutal_input_padding(&theme);
        let hover_border = brutal_border_color(&theme);
        let active_border = brutal_input_focus_border(&theme);
        drop(theme);

        let value_len = state.read(cx).value.chars().count();
        let masked: String = std::iter::repeat_n(mask_char, value_len).collect();

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color,
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
            value_override: Some(masked),
        };

        let base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_3()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(px(BRUTAL_RADIUS))
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .child(inner)
            .track_focus(&focus_handle);

        let keyed = wire_input_keyboard(
            base,
            state.clone(),
            focus_handle.clone(),
            disabled,
            on_submit,
        );
        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .into_any_element()
    }
}

// =====================================================================
// NumberInput
// =====================================================================

pub use yororen_ui_core::renderer::number_input::{NumberInputRenderState, NumberInputRenderer};

pub struct BrutalNumberInputRenderer;

impl NumberInputRenderer for BrutalNumberInputRenderer {
    fn compose(&self, props: &NumberInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        let placeholder_str = props.placeholder.clone();
        let disabled = props.disabled;
        let on_change = props.on_change.clone();
        let on_change_for_state = on_change.clone();
        let on_change_for_dec = on_change.clone();
        let on_change_for_inc = on_change.clone();
        let on_increment = props.on_increment.clone();
        let on_decrement = props.on_decrement.clone();
        let value = props.value;
        let step = props.step;
        let min = props.min;
        let max = props.max;

        let state = window.use_keyed_state(props.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        if state.read(cx).value.is_empty() {
            state.update(cx, |s, _cx| {
                s.value = format!("{}", value);
                s.caret = s.value.len();
            });
        }
        state.update(cx, |s, _cx| {
            s.placeholder = SharedString::from(placeholder_str);
            s.on_change = Some(Arc::new(
                move |new_value: &str, window: &mut Window, cx: &mut App| {
                    if let Some(cb) = on_change_for_state.as_ref() {
                        let parsed = new_value.parse::<f64>().unwrap_or(value);
                        cb(parsed, window, cx);
                    }
                },
            ));
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);
        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let theme = cx.theme().clone();
        let bg = brutal_input_bg(disabled, &theme);
        let border_color = if focused {
            brutal_input_focus_border(&theme)
        } else {
            brutal_input_border(disabled, &theme)
        };
        let text_color = brutal_input_text_color(disabled, &theme);
        let hint_color = brutal_input_hint_color(&theme);
        let min_h = px(theme
            .get_number("tokens.control.number_input.min_height")
            .unwrap_or(42.0) as f32);
        let padding: Edges<Pixels> = {
            let h = theme
                .get_number("tokens.control.number_input.horizontal_padding")
                .unwrap_or(12.0) as f32;
            let v = theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(10.0) as f32;
            Edges::symmetric(px(h), px(v))
        };
        let stepper_size = px(theme
            .get_number("tokens.control.number_input.stepper_button_size")
            .unwrap_or(32.0) as f32);
        let hover_border = brutal_border_color(&theme);
        let active_border = brutal_input_focus_border(&theme);
        drop(theme);

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color,
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
            value_override: None,
        };

        let base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_3()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(px(BRUTAL_RADIUS))
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .track_focus(&focus_handle);
        let keyed = wire_input_keyboard(base, state.clone(), focus_handle.clone(), disabled, None);

        let state_for_dec = state.clone();
        let state_for_inc = state.clone();
        let on_inc_clone = on_increment.clone();
        let on_dec_clone = on_decrement.clone();

        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .child(
                div()
                    .size(stepper_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("−")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        let next = value - step;
                        let clamped = match min {
                            Some(m) => next.max(m),
                            None => next,
                        };
                        let new_text = format!("{}", clamped);
                        state_for_dec.update(cx, |s, cx| {
                            s.value = new_text.clone();
                            s.caret = new_text.len();
                            s.selection_start = new_text.len();
                            s.selection_end = new_text.len();
                            cx.notify();
                        });
                        if let Some(cb) = on_change_for_dec.as_ref() {
                            cb(clamped, window, cx);
                        }
                        if let Some(cb) = on_dec_clone.as_ref() {
                            cb(clamped, window, cx);
                        }
                    }),
            )
            .child(
                div()
                    .size(stepper_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("+")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        let next = value + step;
                        let clamped = match max {
                            Some(m) => next.min(m),
                            None => next,
                        };
                        let new_text = format!("{}", clamped);
                        state_for_inc.update(cx, |s, cx| {
                            s.value = new_text.clone();
                            s.caret = new_text.len();
                            s.selection_start = new_text.len();
                            s.selection_end = new_text.len();
                            cx.notify();
                        });
                        if let Some(cb) = on_change_for_inc.as_ref() {
                            cb(clamped, window, cx);
                        }
                        if let Some(cb) = on_inc_clone.as_ref() {
                            cb(clamped, window, cx);
                        }
                    }),
            )
            .into_any_element()
    }
}

// =====================================================================
// FilePathInput
// =====================================================================

pub use yororen_ui_core::renderer::file_path_input::{
    FilePathInputRenderState, FilePathInputRenderer,
};

pub struct BrutalFilePathInputRenderer;

impl FilePathInputRenderer for BrutalFilePathInputRenderer {
    fn compose(&self, props: &FilePathInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        let placeholder_str = props.placeholder.clone();
        let disabled = props.disabled;
        let on_change = props.on_change.clone();
        let on_browse = props.on_browse.clone();

        let state = window.use_keyed_state(props.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = SharedString::from(placeholder_str);
            s.on_change = on_change.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);
        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let theme = cx.theme().clone();
        let bg = brutal_input_bg(disabled, &theme);
        let border_color = if focused {
            brutal_input_focus_border(&theme)
        } else {
            brutal_input_border(disabled, &theme)
        };
        let text_color = brutal_input_text_color(disabled, &theme);
        let hint_color = brutal_input_hint_color(&theme);
        let button_bg = theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER);
        let button_fg = theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER);
        let button_hover_bg = theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER);
        let min_h = px(theme
            .get_number("tokens.control.file_path_input.min_height")
            .unwrap_or(42.0) as f32);
        let padding: Edges<Pixels> = {
            let h = theme
                .get_number("tokens.control.file_path_input.horizontal_padding")
                .unwrap_or(12.0) as f32;
            let v = theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(10.0) as f32;
            Edges::symmetric(px(h), px(v))
        };
        let action_gap = px(theme
            .get_number("tokens.control.file_path_input.action_gap")
            .unwrap_or(8.0) as f32);
        let icon_size = px(theme
            .get_number("tokens.control.file_path_input.icon_size")
            .unwrap_or(20.0) as f32);
        let hover_border = brutal_border_color(&theme);
        let active_border = brutal_input_focus_border(&theme);
        drop(theme);

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color,
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
            value_override: None,
        };

        let base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_3()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(px(BRUTAL_RADIUS))
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .gap(action_gap)
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .track_focus(&focus_handle);

        let keyed = wire_input_keyboard(base, state.clone(), focus_handle.clone(), disabled, None);

        let on_browse_clone = on_browse.clone();
        let window_handle = window.window_handle();
        let on_change_for_async = state.read(cx).on_change.clone();
        let state_for_browse = state.clone();

        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(
                icon(
                    "brutal-file-path-input-leading-icon",
                    IconSource::Builtin("folder".into()),
                    cx,
                )
                .size(icon_size)
                .color(text_color)
                .render(cx),
            )
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .child(
                div()
                    .id("brutal-file-path-input-browse")
                    .size(icon_size)
                    .bg(button_bg)
                    .rounded(px(BRUTAL_RADIUS))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(button_fg)
                    .hover(|s| s.bg(button_hover_bg))
                    .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
                        if disabled {
                            return;
                        }
                        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
                            files: true,
                            directories: false,
                            multiple: false,
                            prompt: Some("Select a file".into()),
                        });
                        let on_change_for_async = on_change_for_async.clone();
                        let state_for_change = state_for_browse.clone();
                        let on_browse_cb = on_browse_clone.clone();
                        cx.spawn(async move |async_cx| {
                            let picked = match receiver.await {
                                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                                _ => None,
                            };
                            if let Some(path) = picked {
                                let path_str = path.to_string_lossy().to_string();
                                let state_for_change = state_for_change.clone();
                                let on_browse_for_async = on_browse_cb.clone();
                                let _ =
                                    async_cx.update_window(window_handle, move |_, window, cx| {
                                        state_for_change.update(cx, |s, cx| {
                                            s.value = path_str.clone();
                                            let end = s.value.len();
                                            s.caret = end;
                                            s.selection_start = end;
                                            s.selection_end = end;
                                            cx.notify();
                                        });
                                        if let Some(cb) = on_change_for_async.as_ref() {
                                            cb(&path_str, window, cx);
                                        }
                                        if let Some(cb) = on_browse_for_async.as_ref() {
                                            cb(&path_str, window, cx);
                                        }
                                    });
                            }
                        })
                        .detach();
                    })
                    .child(
                        icon(
                            "brutal-file-path-input-browse-icon",
                            IconSource::Builtin("file".into()),
                            cx,
                        )
                        .size(icon_size)
                        .color(button_fg)
                        .render(cx),
                    ),
            )
            .into_any_element()
    }
}

// =====================================================================
// SearchInput
// =====================================================================

pub use yororen_ui_core::renderer::search_input::{SearchInputRenderState, SearchInputRenderer};

pub struct BrutalSearchInputRenderer;

impl SearchInputRenderer for BrutalSearchInputRenderer {
    fn compose(&self, props: &SearchInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        let placeholder_str = props.placeholder.clone();
        let disabled = props.disabled;
        let on_change = props.on_change.clone();
        let on_submit = props.on_submit.clone();
        let on_clear = props.on_clear.clone();

        let state = window.use_keyed_state(props.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = SharedString::from(placeholder_str);
            s.on_change = on_change.clone();
            s.on_submit = on_submit.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);
        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let theme = cx.theme().clone();
        let bg = brutal_input_bg(disabled, &theme);
        let border_color = if focused {
            brutal_input_focus_border(&theme)
        } else {
            brutal_input_border(disabled, &theme)
        };
        let text_color = brutal_input_text_color(disabled, &theme);
        let hint_color = brutal_input_hint_color(&theme);
        let icon_color = brutal_input_hint_color(&theme);
        let min_h = px(theme
            .get_number("tokens.control.search_input.min_height")
            .unwrap_or(42.0) as f32);
        let padding: Edges<Pixels> = {
            let h = theme
                .get_number("tokens.control.search_input.horizontal_padding")
                .unwrap_or(12.0) as f32;
            let v = theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(10.0) as f32;
            Edges::symmetric(px(h), px(v))
        };
        let input_gap = px(theme
            .get_number("tokens.control.search_input.input_gap")
            .unwrap_or(8.0) as f32);
        let icon_size = px(theme
            .get_number("tokens.control.search_input.icon_size")
            .unwrap_or(20.0) as f32);
        let hover_border = brutal_border_color(&theme);
        let active_border = brutal_input_focus_border(&theme);
        drop(theme);

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color,
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
            value_override: None,
        };

        let base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_3()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(px(BRUTAL_RADIUS))
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .gap(input_gap)
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .track_focus(&focus_handle);

        let state_for_escape = state.clone();
        let on_change_for_escape = on_change.clone();
        let keyed = wire_input_keyboard(
            base,
            state.clone(),
            focus_handle.clone(),
            disabled,
            on_submit,
        )
        .on_action(move |_: &Escape, _window, cx| {
            if disabled {
                return;
            }
            let before = state_for_escape.read(cx).value.clone();
            state_for_escape.update(cx, |s, cx| {
                s.value.clear();
                s.caret = 0;
                s.selection_start = 0;
                s.selection_end = 0;
                cx.notify();
            });
            if let Some(cb) = on_change_for_escape.as_ref() {
                let after = state_for_escape.read(cx).value.clone();
                if before != after {
                    cb(&after, _window, cx);
                }
            }
        });

        let state_for_clear = state.clone();
        let on_change_for_clear = on_change.clone();
        let on_clear_clone = on_clear.clone();

        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(
                icon(
                    "brutal-search-input-leading-icon",
                    IconSource::Builtin("search".into()),
                    cx,
                )
                .size(icon_size)
                .color(text_color)
                .render(cx),
            )
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .when(!state_for_clear.read(cx).value.is_empty(), |d| {
                d.child(
                    div()
                        .id("brutal-search-input-clear")
                        .size(icon_size)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(icon_color)
                        .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                            state_for_clear.update(cx, |s, cx| {
                                s.value.clear();
                                s.caret = 0;
                                s.selection_start = 0;
                                s.selection_end = 0;
                                cx.notify();
                            });
                            if let Some(cb) = on_change_for_clear.as_ref() {
                                cb("", window, cx);
                            }
                            if let Some(cb) = on_clear_clone.as_ref() {
                                cb(window, cx);
                            }
                        })
                        .child(
                            icon(
                                "brutal-search-input-clear-icon",
                                IconSource::Builtin("close".into()),
                                cx,
                            )
                            .size(icon_size)
                            .color(icon_color)
                            .render(cx),
                        ),
                )
            })
            .into_any_element()
    }
}

// =====================================================================
// Select (un-migrated trait — keeps the old helper-rich impl)
// =====================================================================

pub use yororen_ui_core::renderer::select::{SelectRenderState, SelectRenderer};

pub struct BrutalSelectRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalSelectRenderer {
    pub fn bg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(state.disabled, theme)
    }
    pub fn border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(state.disabled, theme)
    }
    pub fn focus_border(&self, _: &SelectRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    pub fn fg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else if state.has_value {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn hint_color(&self, _: &SelectRenderState, theme: &Theme) -> Hsla {
        brutal_input_hint_color(theme)
    }
    pub fn min_height(&self, _: &SelectRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.select.min_height")
            .unwrap_or(42.0) as f32)
    }
    pub fn padding(&self, _: &SelectRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.select.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme.get_number("tokens.spacing.tight").unwrap_or(4.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    pub fn border_radius(&self, _: &SelectRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn chevron_rotation(&self, state: &SelectRenderState, _: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

impl SelectRenderer for BrutalSelectRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::select::SelectProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state_read = props.state.read(cx);
        let state = SelectRenderState {
            open: state_read.is_open(),
            disabled: false,
            has_value: state_read.value.is_some(),
            custom_bg: None,
            custom_border: None,
            custom_focus_border: None,
            custom_fg: None,
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let value = state_read.value.clone();
        let options = state_read.options.clone();
        let is_visible = state_read.is_visible();

        // Display label: selected option's label, or the placeholder.
        let display = if let Some(v) = &value {
            options
                .iter()
                .find(|o| &o.value == v)
                .map(|o| o.label.to_string())
                .unwrap_or_else(|| v.to_string())
        } else {
            state_read.placeholder.to_string()
        };

        // Trigger carries the toggle on_click. The wrapper has
        // no click handler, so a click on a dropdown option
        // bubbles harmlessly up the tree.
        //
        // `on_click` is on `StatefulInteractiveElement`, not
        // `InteractiveElement`, so the trigger must be a
        // `Stateful<Div>` (which we get by calling `.id()`
        // first). The headless `apply` later sets the
        // wrapper's id to the main `props.id`; the trigger
        // keeps its own sub-id so the user can identify it.
        let state_for_toggle = props.state.clone();
        let mut trigger: Stateful<gpui::Div> = gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .border_2()
            .border_color(border)
            .text_color(fg)
            .px(pad.left)
            .py(pad.top)
            .min_h(h)
            .rounded(r)
            .cursor(CursorStyle::PointingHand)
            .child(display)
            .id("brutal-select-trigger");
        trigger = trigger.on_click(move |_ev, _window, cx| {
            state_for_toggle.update(cx, |s, _cx| s.toggle());
        });

        let mut outer = gpui::div().relative().child(trigger);

        if is_visible && !options.is_empty() {
            let h_f32: f32 = h.into();
            let state_for_close = props.state.clone();
            let mut dropdown: Stateful<gpui::Div> = gpui::div()
                .id("brutal-select-dropdown")
                .absolute()
                .top(px(h_f32 + 4.0))
                .left_0()
                .right_0()
                .bg(theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER))
                .border_2()
                .border_color(border)
                .rounded(r)
                .p(px(4.))
                .flex_col()
                .gap(px(2.))
                // `.occlude_mouse()` makes the dropdown's hitbox
                // block mouse events from reaching elements
                // painted behind it. This is what stops a click
                // on an option from also firing on the cell
                // directly below the popover.
                .occlude()
                // `on_mouse_down_out` fires whenever the user
                // presses the mouse *outside* the dropdown's
                // bounds (including clicks in the next cell,
                // the toolbar, anywhere in the window) and is
                // the conventional way of wiring "click outside
                // dismisses".
                .on_mouse_down_out(move |_ev, _window, cx| {
                    state_for_close.update(cx, |s, _cx| s.close());
                });

            for (i, opt) in options.iter().enumerate() {
                let opt_value = opt.value.clone();
                let opt_label = opt.label.to_string();
                let state_for_opt = props.state.clone();
                let is_selected = value.as_ref() == Some(&opt.value);
                let item_bg = if is_selected {
                    theme
                        .get_color("action.primary.bg")
                        .unwrap_or(BRUTAL_BORDER)
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let hover_bg = theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER);
                let item_fg = if is_selected {
                    theme
                        .get_color("action.primary.fg")
                        .unwrap_or(BRUTAL_BORDER)
                } else {
                    theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
                };
                let mut item: Stateful<gpui::Div> = gpui::div()
                    .id(ElementId::Name(format!("brutal-select-opt-{}", i).into()))
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(4.))
                    .bg(item_bg)
                    .text_color(item_fg)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |s| s.bg(hover_bg))
                    .child(opt_label);
                item = item.on_click(move |_ev, window, cx| {
                    // Headless data action: `pick` writes
                    // value, closes the dropdown, and fires
                    // `on_change` in one call. We recover
                    // `&mut App` from the `Context` via
                    // `&mut *cx_inner` (the documented
                    // `DerefMut<Target = App>` pattern — see
                    // memory.md "Context<T> → App").
                    state_for_opt.update(cx, |s, cx_inner| {
                        s.pick(opt_value.clone(), window, &mut *cx_inner);
                    });
                });
                dropdown = dropdown.child(item);
            }

            // `gpui::deferred` paints the dropdown after the
            // next sibling cell so it isn't covered. The animation
            // wrapper is absolutely positioned at the top-left of the
            // outer relative container so the dropdown keeps its
            // original `top/left/right` offsets.
            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            outer = outer.child(
                gpui::deferred(gpui::div().absolute().top_0().left_0().right_0().child(
                    AnimatedPresenceElement::new(
                        props.state.clone(),
                        (props.id.clone(), "dropdown"),
                        SlideDirection::Down,
                        distance,
                        gpui::div().child(dropdown),
                    ),
                ))
                .with_priority(1),
            );
        }

        outer
    }
}

// =====================================================================
// ComboBox (un-migrated trait)
// =====================================================================

pub use yororen_ui_core::renderer::combo_box::{ComboBoxRenderState, ComboBoxRenderer};

pub struct BrutalComboBoxRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalComboBoxRenderer {
    pub fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(state.disabled, theme)
    }
    pub fn border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(state.disabled, theme)
    }
    pub fn focus_border(&self, _: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    pub fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else if state.has_value {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn search_bg(&self, _: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, _: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.combo_box.min_height")
            .unwrap_or(42.0) as f32)
    }
    pub fn padding(&self, _: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.combo_box.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme.get_number("tokens.spacing.tight").unwrap_or(4.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    pub fn border_radius(&self, _: &ComboBoxRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl ComboBoxRenderer for BrutalComboBoxRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::combo_box::ComboBoxProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

        let theme = cx.theme().clone();
        let (state, text, value, options, is_open, is_visible, placeholder) = {
            let state_read = props.state.read(cx);
            let state = ComboBoxRenderState {
                open: state_read.is_open(),
                disabled: false,
                has_value: state_read.value.is_some(),
                editable: state_read.editable,
                custom_bg: None,
                custom_border: None,
                custom_focus_border: None,
                custom_fg: None,
            };
            (
                state,
                state_read.text.clone(),
                state_read.value.clone(),
                state_read.options.clone(),
                state_read.is_open(),
                state_read.is_visible(),
                state_read.placeholder.clone(),
            )
        };
        let bg = self.bg(&state, &theme);
        let border = self.border(&state, &theme);
        // The trigger's foreground colour is owned by the
        // embedded text-input; the outer wrapper only carries
        // bg / border / padding / radius.
        let pad = self.padding(&state, &theme);
        let h = self.min_height(&state, &theme);
        let r = self.border_radius(&state, &theme);

        // -------- Text-input trigger --------
        // The combo box trigger is a real text input backed
        // directly by `ComboBoxState.core`. No separate
        // `TextInputState` entity is minted.
        let focus_handle = props.state.read(cx).core.focus_handle();
        let focused = focus_handle.is_focused(window);
        // Selection-only mode (`editable == false`) is
        // display-only: no caret blink, pointing-hand cursor.
        if focused && state.editable {
            start_cursor_blink(props.state.clone(), window, cx);
        } else if state.editable {
            props
                .state
                .update(cx, |s, _cx| s.core.cursor_visible = true);
        } else {
            props
                .state
                .update(cx, |s, _cx| s.core.cursor_visible = false);
        }

        let display_str: String = if !text.is_empty() {
            text.clone()
        } else if let Some(v) = &value {
            options
                .iter()
                .find(|o| &o.value == v)
                .map(|o| o.label.to_string())
                .unwrap_or_else(|| v.to_string())
        } else {
            String::new()
        };

        let text_color = theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER);
        let hint_color = theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER);
        let cursor_color = theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER);
        let selection_color = {
            let c = theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER);
            gpui::hsla(c.h, c.s, c.l, 0.4)
        };

        let ti_element = TextInputElement {
            state: props.state.clone(),
            focus_handle: focus_handle.clone(),
            disabled: false,
            text_color,
            hint_color,
            cursor_color,
            selection_color,
            placeholder,
            value_override: Some(display_str),
        }
        .into_any_element();

        // The chevron is a static triangle, laid out on the
        // right of the text input. Sized to the trigger
        // height, vertically centred.
        let chevron_w = px(20.0);
        let mut trigger: Stateful<gpui::Div> = gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .border_2()
            .border_color(border)
            .px(pad.left)
            .min_h(h)
            .rounded(r)
            .id("brutal-combo-trigger")
            .track_focus(&focus_handle)
            .cursor(if state.editable {
                CursorStyle::IBeam
            } else {
                CursorStyle::PointingHand
            })
            // The text input is the flex child that grows;
            // the chevron is the fixed-size child on the
            // right. Click anywhere on the trigger opens the
            // dropdown (text input's own focus behaviour is
            // orthogonal).
            .child(div().flex_1().min_w(px(0.)).child(ti_element))
            .child(
                div()
                    .w(chevron_w)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(hint_color)
                    .cursor(CursorStyle::PointingHand)
                    .child(if is_open { "▴" } else { "▾" }),
            );
        let combo_state_for_open = props.state.clone();
        trigger = trigger.on_click(move |_ev, _window, cx| {
            combo_state_for_open.update(cx, |s, _cx| s.toggle());
        });

        let trigger = wire_input_keyboard(trigger, props.state.clone(), focus_handle, false, None);

        // -------- Filtered dropdown --------
        // Filter is case-insensitive `contains(label, text)`.
        // When `text` is empty we show every option.
        let needle = text.to_lowercase();
        let filtered: Vec<(usize, &yororen_ui_core::headless::combo_box::ComboBoxOption)> = options
            .iter()
            .enumerate()
            .filter(|(_, opt)| needle.is_empty() || opt.label.to_lowercase().contains(&needle))
            .collect();

        let mut outer = gpui::div().relative().child(trigger);

        if is_visible && !filtered.is_empty() {
            let h_f32: f32 = h.into();
            let state_for_close = props.state.clone();
            let mut dropdown: Stateful<gpui::Div> = gpui::div()
                .id("brutal-combo-dropdown")
                .absolute()
                .top(px(h_f32 + 4.0))
                .left_0()
                .right_0()
                .bg(theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER))
                .border_2()
                .border_color(border)
                .rounded(r)
                .p(px(4.))
                .flex_col()
                .gap(px(2.))
                .occlude()
                .on_mouse_down_out(move |_ev, _window, cx| {
                    state_for_close.update(cx, |s, _cx| s.close());
                });

            for (orig_i, opt) in filtered.iter() {
                let opt_value = opt.value.clone();
                let opt_label = opt.label.to_string();
                let state_for_opt = props.state.clone();
                let is_selected = value.as_ref() == Some(&opt.value);
                let item_bg = if is_selected {
                    theme
                        .get_color("action.primary.bg")
                        .unwrap_or(BRUTAL_BORDER)
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let hover_bg = theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER);
                let item_fg = if is_selected {
                    theme
                        .get_color("action.primary.fg")
                        .unwrap_or(BRUTAL_BORDER)
                } else {
                    theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
                };
                let mut item: Stateful<gpui::Div> = gpui::div()
                    .id(ElementId::Name(
                        format!("brutal-combo-opt-{}", orig_i).into(),
                    ))
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(4.))
                    .bg(item_bg)
                    .text_color(item_fg)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |s| s.bg(hover_bg))
                    .child(opt_label);
                // On pick: headless `pick` writes value
                // (which also resyncs `text` to the label)
                // AND closes the dropdown AND fires
                // `on_change` in one call. The trigger's
                // text_input will re-paint with the label
                // on the next frame.
                item = item.on_click(move |_ev, window, cx| {
                    // Recover `&mut App` from the `Context`
                    // via `&mut *cx_inner` (the documented
                    // `DerefMut<Target = App>` pattern —
                    // see memory.md "Context<T> → App").
                    state_for_opt.update(cx, |s, cx_inner| {
                        s.pick(opt_value.clone(), window, &mut *cx_inner);
                    });
                });
                dropdown = dropdown.child(item);
            }

            // The animation wrapper is absolutely positioned at the
            // top-left of the outer relative container so the dropdown
            // inside keeps its original `top/left/right` offsets.
            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            outer = outer.child(
                gpui::deferred(gpui::div().absolute().top_0().left_0().right_0().child(
                    AnimatedPresenceElement::new(
                        props.state.clone(),
                        (props.id.clone(), "dropdown"),
                        SlideDirection::Down,
                        distance,
                        gpui::div().child(dropdown),
                    ),
                ))
                .with_priority(1),
            );
        }

        outer.into_any_element()
    }
}

// =====================================================================
// KeybindingInput
// =====================================================================

pub use yororen_ui_core::renderer::keybinding_input::{
    KeybindingInputRenderState, KeybindingInputRenderer,
};

pub struct BrutalKeybindingInputRenderer;

impl KeybindingInputRenderer for BrutalKeybindingInputRenderer {
    fn compose(
        &self,
        props: &KeybindingInputProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement {
        let placeholder_str = props.placeholder.clone();
        let disabled = props.disabled;
        let on_change = props.on_change.clone();
        let on_start_capture = props.on_start_capture.clone();
        let on_cancel_capture = props.on_cancel_capture.clone();
        let mode = props.mode;

        let state = window.use_keyed_state(props.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = SharedString::from(placeholder_str);
            s.on_change = on_change.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);
        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let theme = cx.theme().clone();
        let bg = brutal_input_bg(disabled, &theme);
        let border_color = if mode == KeybindingInputMode::Capturing || focused {
            brutal_input_focus_border(&theme)
        } else {
            brutal_input_border(disabled, &theme)
        };
        let kbd_bg = theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER);
        let kbd_fg = theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER);
        let min_h = brutal_input_min_height(&theme);
        let hover_border = brutal_border_color(&theme);
        let active_border = brutal_input_focus_border(&theme);
        drop(theme);

        let base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_3()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(px(BRUTAL_RADIUS))
            .px(px(10.))
            .py(px(4.))
            .flex()
            .items_center()
            .text_color(kbd_fg)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .track_focus(&focus_handle);

        let mut keyed: Stateful<Div> =
            wire_input_keyboard(base, state.clone(), focus_handle.clone(), disabled, None);

        if mode == KeybindingInputMode::Capturing && !disabled {
            let state_for_capture = state.clone();
            let on_change_for_capture = on_change.clone();
            let on_cancel_for_capture = on_cancel_capture.clone();
            keyed = keyed.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let ks = &ev.keystroke;
                if ks.key.as_str() == "escape" {
                    if let Some(cb) = on_cancel_for_capture.as_ref() {
                        cb(window, cx);
                    }
                    return;
                }
                let mut parts: Vec<String> = Vec::new();
                if ks.modifiers.control {
                    parts.push("ctrl".into());
                }
                if ks.modifiers.alt {
                    parts.push("alt".into());
                }
                if ks.modifiers.shift {
                    parts.push("shift".into());
                }
                if ks.modifiers.platform {
                    parts.push("cmd".into());
                }
                if ks.key.is_empty() {
                    return;
                }
                parts.push(ks.key.clone());
                let combo = parts.join("-");
                state_for_capture.update(cx, |s, _cx| {
                    s.value = combo.clone();
                    s.caret = combo.len();
                    s.selection_start = combo.len();
                    s.selection_end = combo.len();
                });
                if let Some(cb) = on_change_for_capture.as_ref() {
                    cb(&combo, window, cx);
                }
            });
        }

        let on_start_clone = on_start_capture.clone();
        let display_text = if mode == KeybindingInputMode::Capturing {
            if state.read(cx).value.is_empty() {
                "Press a key…".to_string()
            } else {
                state.read(cx).value.clone()
            }
        } else if state.read(cx).value.is_empty() {
            "(unset)".to_string()
        } else {
            state.read(cx).value.clone()
        };

        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                if let Some(cb) = on_start_clone.as_ref() {
                    cb(window, cx);
                }
            })
            .child(
                div()
                    .bg(kbd_bg)
                    .rounded(px(BRUTAL_RADIUS))
                    .px(px(8.))
                    .py(px(2.))
                    .text_color(kbd_fg)
                    .child(display_text),
            )
            .into_any_element()
    }
}
