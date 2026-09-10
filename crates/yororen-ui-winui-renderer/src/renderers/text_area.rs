//! `WinUITextAreaRenderer` — default `TextAreaRenderer` impl.

use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, Div, Hsla, InteractiveElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, ParentElement, Pixels, SharedString, Stateful, StatefulInteractiveElement,
    Styled, Window, div, hsla, px,
};

use yororen_ui_core::action_handler;
use yororen_ui_core::headless::text_area::TextAreaProps;
use yororen_ui_core::headless::text_area_element::TextAreaElement;
use yororen_ui_core::headless::text_input::{
    Backspace, Copy, Cut, Delete, End, Enter, Home, Left, Paste, Right, SelectAll, SelectLeft,
    SelectRight, ShowCharacterPalette, TextInputState,
};
use yororen_ui_core::headless::text_input_element::start_cursor_blink;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::renderer::text_area::{TextAreaRenderState, TextAreaRenderer};

use crate::animation::{animated_input_border, set_interaction_hovered, set_interaction_pressed};
use crate::themes::default_font;
use yororen_ui_core::theme::Theme;

pub struct WinUITextAreaRenderer;

// Inherent helpers — *not* part of the `TextAreaRenderer`
// trait surface.
impl WinUITextAreaRenderer {
    pub fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("winui.ctrl_fill_disabled")
                .or_else(|| theme.get_color("surface.sunken"))
                .unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    pub fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    pub fn focus_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    pub fn hover_border(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.ctrl_strong_stroke")
            .or_else(|| theme.get_color("border.muted"))
            .unwrap_or_default()
    }
    pub fn active_border(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else {
            state
                .custom_text_color
                .unwrap_or_else(|| theme.get_color("content.primary").unwrap_or_default())
        }
    }
    pub fn min_height(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.input.text_area_min_h")
            .unwrap_or(0.0) as f32)
    }
    pub fn padding(&self, _state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels> {
        crate::themes::input_field_padding(theme)
    }
    pub fn border_radius(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

impl TextAreaRenderer for WinUITextAreaRenderer {
    fn compose(&self, props: &TextAreaProps, cx: &mut App, window: &mut Window) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

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
        let render_state = TextAreaRenderState {
            disabled,
            focused,
            has_custom_bg: props.has_custom_bg,
            custom_bg: props.custom_bg,
            custom_border: props.custom_border,
            has_custom_focus_border: props.has_custom_focus_border,
            custom_focus_border: props.custom_focus_border,
            custom_text_color: props.custom_text_color,
        };
        let bg = self.bg(&render_state, &theme);
        let border_color = if focused {
            self.focus_border(&render_state, &theme)
        } else {
            self.border(&render_state, &theme)
        };
        let text_color = self.text_color(&render_state, &theme);
        let min_h = self.min_height(&render_state, &theme);
        let padding = self.padding(&render_state, &theme);
        let radius = self.border_radius(&render_state, &theme);
        let side_border = theme
            .get_color("winui.ctrl_stroke")
            .unwrap_or_else(|| self.border(&render_state, &theme));
        let bottom_rest = theme
            .get_color("winui.ctrl_strong_stroke")
            .unwrap_or_else(|| self.border(&render_state, &theme));
        let bottom_focused = theme
            .get_color("winui.accent")
            .unwrap_or_else(|| self.border(&render_state, &theme));
        let bg_hover = theme.get_color("winui.ctrl_fill_hover").unwrap_or(bg);
        let bg_focused = theme
            .get_color("winui.ctrl_fill_input_active")
            .or_else(|| theme.get_color("surface.sunken"))
            .unwrap_or(bg);
        let hint_color = theme
            .get_color("winui.text_secondary")
            .or_else(|| theme.get_color("content.secondary"))
            .unwrap_or_default();
        let cursor_color = if props.has_custom_focus_border {
            props
                .custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        };
        let focus_color = theme.get_color("border.focus").unwrap_or_default();
        let selection_color = hsla(focus_color.color.hue.into_degrees() / 360.0, focus_color.saturation, focus_color.lightness, 0.25);
        let font = default_font(&theme);
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

        let mut base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .pl(padding.left)
            .pr(padding.right)
            .pt(padding.top)
            .pb(padding.bottom)
            .font_family(font.clone())
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .child(inner)
            .track_focus(&focus_handle);

        if !disabled {
            base = base
                .on_hover({
                    let id = props.id.clone();
                    move |hovered, _win, cx| set_interaction_hovered(cx, id.clone(), *hovered)
                })
                .on_mouse_down(MouseButton::Left, {
                    let id = props.id.clone();
                    move |_, _win, cx| set_interaction_pressed(cx, id.clone(), true)
                })
                .on_mouse_up(MouseButton::Left, {
                    let id = props.id.clone();
                    move |_, _win, cx| set_interaction_pressed(cx, id.clone(), false)
                });
        }

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

        animated_input_border(
            keyed,
            props.id.clone(),
            props.id.clone(),
            side_border,
            bottom_rest,
            bottom_focused,
            bg,
            bg_hover,
            bg_focused,
            focused,
        )
    }
}

pub fn arc_text_area<T: TextAreaRenderer + 'static>(r: T) -> Arc<dyn TextAreaRenderer> {
    Arc::new(r)
}
