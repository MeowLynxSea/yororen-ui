//! `WinUINumberInputRenderer` — default `NumberInputRenderer` impl.

use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, Div, Hsla, InteractiveElement, MouseButton, ParentElement, Pixels,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};

use yororen_ui_core::headless::number_input::NumberInputProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::headless::text_input_element::{
    TextInputElement, start_cursor_blink, wire_input_keyboard,
};
use yororen_ui_core::renderer::number_input::{NumberInputRenderState, NumberInputRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::animation::{
    animated_input_border, set_interaction_hovered, set_interaction_pressed,
};
use crate::themes::default_font;

pub struct WinUINumberInputRenderer;

// Inherent helpers — *not* part of the `NumberInputRenderer`
// trait surface.
impl WinUINumberInputRenderer {
    pub fn bg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn focus_border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn hover_border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.ctrl_strong_stroke")
            .or_else(|| theme.get_color("border.muted"))
            .unwrap_or_default()
    }
    pub fn active_border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.number_input.min_height")
            .unwrap_or(0.0) as f32)
    }
    pub fn padding(&self, _state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.number_input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    pub fn stepper_button_size(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.number_input.stepper_button_size")
            .unwrap_or(0.0) as f32)
    }
    pub fn border_radius(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

impl NumberInputRenderer for WinUINumberInputRenderer {
    fn compose(&self, props: &NumberInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

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
        let render_state = NumberInputRenderState {
            disabled,
            focused,
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
        let text_color = theme.get_color("content.primary").unwrap_or_default();
        let hint_color = theme.get_color("content.tertiary").unwrap_or_default();
        let min_h = self.min_height(&render_state, &theme);
        let padding = self.padding(&render_state, &theme);
        let radius = self.border_radius(&render_state, &theme);
        let stepper_size = self.stepper_button_size(&render_state, &theme);
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

        let mut base: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .font_family(font.clone())
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .track_focus(&focus_handle);

        if focused {
            base = base.border_2();
        }

        if !disabled {
            base = base
                .on_hover({
                    let id = props.id.clone();
                    move |hovered, _win, cx| set_interaction_hovered(cx, id.clone(), *hovered)
                })
                .on_mouse_down(
                    MouseButton::Left,
                    {
                        let id = props.id.clone();
                        move |_, _win, cx| set_interaction_pressed(cx, id.clone(), true)
                    },
                )
                .on_mouse_up(
                    MouseButton::Left,
                    {
                        let id = props.id.clone();
                        move |_, _win, cx| set_interaction_pressed(cx, id.clone(), false)
                    },
                );
        }

        let keyed = wire_input_keyboard(base, state.clone(), focus_handle.clone(), disabled, None);

        let state_for_dec = state.clone();
        let state_for_inc = state.clone();
        let on_inc_clone = on_increment.clone();
        let on_dec_clone = on_decrement.clone();

        let assembled = keyed
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .child(
                div()
                    .id("number-input-decrement")
                    .size(stepper_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor(CursorStyle::PointingHand)
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
                    })
                    .child("−"),
            )
            .child(
                div()
                    .id("number-input-increment")
                    .size(stepper_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor(CursorStyle::PointingHand)
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
                    })
                    .child("+"),
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

pub fn arc_number_input<T: NumberInputRenderer + 'static>(r: T) -> Arc<dyn NumberInputRenderer> {
    Arc::new(r)
}
