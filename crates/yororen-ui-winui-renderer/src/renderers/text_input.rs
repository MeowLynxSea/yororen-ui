//! `WinUITextInputRenderer` — default `TextInputRenderer` impl.
//!
//! `compose` owns the entire visual pipeline: it mints the
//! input state via `window.use_keyed_state`, reads tokens
//! from the theme, builds the wrapper div, layers the inner
//! `TextInputElement`, and wires the 14-action keymap. Data
//! flow is one-way: headless just calls `compose` and returns
//! the resulting `AnyElement`.

use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, Div, Hsla, InteractiveElement, MouseButton, ParentElement,
    Pixels, SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, hsla, px,
};

use yororen_ui_core::headless::text_input::{TextInputProps, TextInputState};
use yororen_ui_core::headless::text_input_element::{
    TextInputElement, start_cursor_blink, wire_input_keyboard,
};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::renderer::text_input::{TextInputRenderState, TextInputRenderer};
use yororen_ui_core::theme::Theme;

use crate::animation::{animated_input_border, set_interaction_hovered, set_interaction_pressed};
use crate::themes::default_font;

pub struct WinUITextInputRenderer;

// Inherent helpers — *not* part of the `TextInputRenderer`
// trait surface. They exist so `compose` can stay readable
// and so other text-input-like renderers can share the same
// palette decisions.
impl WinUITextInputRenderer {
    pub fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else if state.has_custom_bg {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    pub fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
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
    pub fn focus_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_focus_border {
            state
                .custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        }
    }
    pub fn hover_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        // When the caller passes `has_custom_border: true`
        // (e.g. the combo_box embedding this text input in
        // its trigger) we want hover to keep the same custom
        // border, not snap to the theme's `border.muted`. The
        // default light hover otherwise introduces a visible
        // ring the caller tried to suppress.
        if state.has_custom_border {
            return self.border(state, theme);
        }
        theme
            .get_color("winui.ctrl_strong_stroke")
            .or_else(|| theme.get_color("border.muted"))
            .unwrap_or_default()
    }
    pub fn active_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        // Same as `hover_border`: when the caller suppressed
        // the default border, also suppress the active state
        // so a mouse-down on the input doesn't briefly
        // re-introduce it.
        if state.has_custom_border {
            return self.border(state, theme);
        }
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.custom_text_color.is_some() {
            state.custom_text_color.unwrap()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    pub fn hint_color(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    pub fn cursor_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        // With `has_custom_focus_border: true` the focus border
        // is the caller's transparent / themed override, so
        // it would make the caret invisible. Fall back to the
        // text colour (or the explicit `custom_text_color`)
        // so the cursor stays visible.
        if state.has_custom_focus_border {
            return self.text_color(state, theme);
        }
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn selection_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        let c = if state.has_custom_focus_border {
            self.text_color(state, theme)
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        };
        hsla(c.h, c.s, c.l, 0.25)
    }
    pub fn min_height(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.input.min_height")
            .unwrap_or(0.0) as f32)
    }
    pub fn padding(&self, _state: &TextInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    pub fn border_radius(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn disabled_opacity(&self, state: &TextInputRenderState, _theme: &Theme) -> f32 {
        if state.disabled { 0.6 } else { 1.0 }
    }

    fn render_state(props: &TextInputProps, focused: bool) -> TextInputRenderState {
        TextInputRenderState {
            disabled: props.disabled,
            focused,
            has_custom_bg: props.has_custom_bg,
            has_custom_border: props.has_custom_border,
            has_custom_focus_border: props.has_custom_focus_border,
            custom_bg: props.custom_bg,
            custom_border: props.custom_border,
            custom_focus_border: props.custom_focus_border,
            custom_text_color: props.custom_text_color,
        }
    }
}

impl TextInputRenderer for WinUITextInputRenderer {
    fn compose(&self, props: &TextInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

        let placeholder_str = props.placeholder.clone();
        let max_length = props.max_length;
        let disabled = props.disabled;
        let on_change = props.on_change.clone();
        let on_submit = props.on_submit.clone();
        let initial_value = props.initial_value.clone();

        // Mint / reuse the per-element state.
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

        // Read tokens (snapshot the values before we hand the
        // theme arc back to the borrow checker).
        let theme = cx.theme().clone();
        let render_state = Self::render_state(props, focused);
        let bg = self.bg(&render_state, &theme);
        let border_color = if focused {
            self.focus_border(&render_state, &theme)
        } else {
            self.border(&render_state, &theme)
        };
        let text_color = self.text_color(&render_state, &theme);
        let hint_color = self.hint_color(&render_state, &theme);
        let cursor_color = self.cursor_color(&render_state, &theme);
        let selection_color = self.selection_color(&render_state, &theme);
        let min_h = self.min_height(&render_state, &theme);
        let padding = self.padding(&render_state, &theme);
        let radius = self.border_radius(&render_state, &theme);
        let opacity = self.disabled_opacity(&render_state, &theme);
        // Fluent TextBox colors: thin side border, brighter bottom
        // underline, rest/hover/focused backgrounds.
        let side_border = theme.get_color("winui.ctrl_stroke").unwrap_or(border_color);
        let bottom_rest = theme
            .get_color("winui.ctrl_strong_stroke")
            .unwrap_or(border_color);
        let bottom_focused = theme.get_color("winui.accent").unwrap_or(border_color);
        let bg_hover = theme.get_color("winui.ctrl_fill_hover").unwrap_or(bg);
        let bg_focused = theme.get_color("surface.sunken").unwrap_or(bg);
        let font = default_font(&theme);
        drop(theme);

        // Build the wrapper.
        let mut styled: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .opacity(opacity)
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

        // The old `border_2()` focus treatment is replaced by the
        // Fluent underline in `animated_input_border`; the wrapper
        // still needs its ElementId-based interactivity shell.

        if !disabled {
            styled = styled
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

        // Inner text-painting element.
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
        let with_child = styled.child(inner);

        // Wire the keymap.
        let keyed = wire_input_keyboard(
            with_child,
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

pub fn arc_text_input<T: TextInputRenderer + 'static>(r: T) -> Arc<dyn TextInputRenderer> {
    Arc::new(r)
}
