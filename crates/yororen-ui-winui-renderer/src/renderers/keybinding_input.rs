//! `WinUIKeybindingInputRenderer` — default `KeybindingInputRenderer` impl.

use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, Div, Hsla, InteractiveElement, KeyDownEvent, MouseButton,
    ParentElement, Pixels, SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div,
    px,
};

use yororen_ui_core::headless::keybinding_input::{KeybindingInputMode, KeybindingInputProps};
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::headless::text_input_element::{start_cursor_blink, wire_input_keyboard};
use yororen_ui_core::renderer::keybinding_input::{
    KeybindingInputRenderState, KeybindingInputRenderer,
};
use yororen_ui_core::theme::Theme;

use crate::animation::{animated_input_border, set_interaction_hovered, set_interaction_pressed};
use crate::themes::default_font;

pub struct WinUIKeybindingInputRenderer;

// Inherent helpers — *not* part of the `KeybindingInputRenderer`
// trait surface.
impl WinUIKeybindingInputRenderer {
    pub fn bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn focus_border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn hover_border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    pub fn active_border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn kbd_bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn kbd_fg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.input.min_height")
            .unwrap_or(0.0) as f32)
    }
    pub fn border_radius(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

impl KeybindingInputRenderer for WinUIKeybindingInputRenderer {
    fn compose(
        &self,
        props: &KeybindingInputProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

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
        let render_state = KeybindingInputRenderState {
            capturing: mode == KeybindingInputMode::Capturing,
            disabled,
            custom_bg: props.custom_bg,
            custom_border: props.custom_border,
            custom_focus_border: props.custom_focus_border,
            custom_fg: props.custom_text_color,
        };
        let bg = self.bg(&render_state, &theme);
        let border_color = if focused {
            self.focus_border(&render_state, &theme)
        } else {
            self.border(&render_state, &theme)
        };
        let kbd_bg = self.kbd_bg(&render_state, &theme);
        let kbd_fg = self.kbd_fg(&render_state, &theme);
        let min_h = self.min_height(&render_state, &theme);
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
        let bg_focused = theme.get_color("surface.sunken").unwrap_or(bg);
        let font = default_font(&theme);
        drop(theme);

        let mut base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .px(px(10.))
            .py(px(4.))
            .flex()
            .items_center()
            .font_family(font.clone())
            .text_color(kbd_fg)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
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

        // In capturing mode we bypass `wire_input_keyboard` so that
        // the text-input action layer does not consume `Escape`
        // before we can cancel capture. All other input modes keep
        // the standard keyboard wiring.
        let mut keyed: Stateful<Div> = if mode == KeybindingInputMode::Capturing && !disabled {
            base
        } else {
            wire_input_keyboard(base, state.clone(), focus_handle.clone(), disabled, None)
        };

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

        let assembled = keyed
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                if let Some(cb) = on_start_clone.as_ref() {
                    cb(window, cx);
                }
            })
            .child(
                div()
                    .bg(kbd_bg)
                    .rounded(px(4.0))
                    .px(px(8.))
                    .py(px(2.))
                    .text_color(kbd_fg)
                    .child(display_text),
            );

        animated_input_border(
            assembled,
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

pub fn arc_keybinding_input<T: KeybindingInputRenderer + 'static>(
    r: T,
) -> Arc<dyn KeybindingInputRenderer> {
    Arc::new(r)
}
