//! `WinUISearchInputRenderer` — default `SearchInputRenderer` impl.

use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, Div, Hsla, InteractiveElement, MouseButton, ParentElement,
    Pixels, SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use yororen_ui_core::headless::icon::{IconSource, icon};
use yororen_ui_core::headless::search_input::SearchInputProps;
use yororen_ui_core::headless::text_input::{Escape, TextInputState};
use yororen_ui_core::headless::text_input_element::{
    TextInputElement, start_cursor_blink, wire_input_keyboard,
};
use yororen_ui_core::renderer::search_input::{SearchInputRenderState, SearchInputRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::animation::{animated_input_border, set_interaction_hovered, set_interaction_pressed};
use crate::themes::default_font;

pub struct WinUISearchInputRenderer;

// Inherent helpers — *not* part of the `SearchInputRenderer`
// trait surface.
impl WinUISearchInputRenderer {
    pub fn bg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn focus_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn hover_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.ctrl_strong_stroke")
            .or_else(|| theme.get_color("border.muted"))
            .unwrap_or_default()
    }
    pub fn active_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn icon_color(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    pub fn fg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.search_input.min_height")
            .unwrap_or(0.0) as f32)
    }
    pub fn padding(&self, _state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.search_input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    pub fn border_radius(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn input_gap(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.search_input.input_gap")
            .unwrap_or(0.0) as f32)
    }
    pub fn icon_size(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.search_input.icon_size")
            .unwrap_or(0.0) as f32)
    }
}

impl SearchInputRenderer for WinUISearchInputRenderer {
    fn compose(&self, props: &SearchInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

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
        let render_state = SearchInputRenderState {
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
        let text_color = self.fg(&render_state, &theme);
        let hint_color = theme.get_color("content.tertiary").unwrap_or_default();
        let icon_color = self.icon_color(&render_state, &theme);
        let min_h = self.min_height(&render_state, &theme);
        let padding = self.padding(&render_state, &theme);
        let radius = self.border_radius(&render_state, &theme);
        let input_gap = self.input_gap(&render_state, &theme);
        let icon_size = self.icon_size(&render_state, &theme);
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
            .gap(input_gap)
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
                .on_mouse_down(MouseButton::Left, {
                    let id = props.id.clone();
                    move |_, _win, cx| set_interaction_pressed(cx, id.clone(), true)
                })
                .on_mouse_up(MouseButton::Left, {
                    let id = props.id.clone();
                    move |_, _win, cx| set_interaction_pressed(cx, id.clone(), false)
                });
        }

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

        let assembled: Stateful<Div> = keyed
            .child(
                icon(
                    "search-input-leading-icon",
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
                        .id("search-input-clear")
                        .size(icon_size)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(icon_color)
                        .cursor(CursorStyle::PointingHand)
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
                                "search-input-clear-icon",
                                IconSource::Builtin("close".into()),
                                cx,
                            )
                            .size(icon_size)
                            .color(icon_color)
                            .render(cx),
                        ),
                )
            });

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

pub fn arc_search_input<T: SearchInputRenderer + 'static>(r: T) -> Arc<dyn SearchInputRenderer> {
    Arc::new(r)
}
