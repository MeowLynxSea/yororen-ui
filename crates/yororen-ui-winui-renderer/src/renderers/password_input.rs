//! `WinUIPasswordInputRenderer` — default `PasswordInputRenderer` impl.

use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, Div, Hsla, InteractiveElement, MouseButton, ParentElement,
    Pixels, SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};

use yororen_ui_core::headless::password_input::PasswordInputProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::headless::text_input_element::{
    TextInputElement, start_cursor_blink, wire_input_keyboard,
};
use yororen_ui_core::renderer::password_input::{PasswordInputRenderState, PasswordInputRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::animation::{animated_input_border, set_interaction_hovered, set_interaction_pressed};
use crate::themes::default_font;

pub struct WinUIPasswordInputRenderer;

// Inherent helpers — *not* part of the `PasswordInputRenderer`
// trait surface.
impl WinUIPasswordInputRenderer {
    pub fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("winui.ctrl_fill_disabled")
                .or_else(|| theme.get_color("surface.sunken"))
                .unwrap_or_default()
        } else if state.has_custom_bg {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    pub fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else if state.has_custom_border {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        } else {
            theme.get_color("border.default").unwrap_or_default()
        }
    }
    pub fn focus_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_focus_border {
            state
                .custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        }
    }
    pub fn hover_border(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.ctrl_strong_stroke")
            .or_else(|| theme.get_color("border.muted"))
            .unwrap_or_default()
    }
    pub fn active_border(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else {
            state
                .custom_fg
                .unwrap_or_else(|| theme.get_color("content.primary").unwrap_or_default())
        }
    }
    pub fn min_height(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.input.min_height")
            .unwrap_or(0.0) as f32)
    }
    pub fn padding(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels> {
        crate::themes::input_field_padding(theme)
    }
    pub fn border_radius(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

impl PasswordInputRenderer for WinUIPasswordInputRenderer {
    fn compose(&self, props: &PasswordInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

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
        let render_state = PasswordInputRenderState {
            disabled,
            focused,
            has_custom_bg: props.has_custom_bg,
            has_custom_border: props.has_custom_border,
            has_custom_focus_border: props.has_custom_focus_border,
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
        let text_color = self.fg(&render_state, &theme);
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
        let font = default_font(&theme);
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

        let keyed = wire_input_keyboard(
            base,
            state.clone(),
            focus_handle.clone(),
            disabled,
            on_submit,
        );

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

pub fn arc_password_input<T: PasswordInputRenderer + 'static>(
    r: T,
) -> Arc<dyn PasswordInputRenderer> {
    Arc::new(r)
}
