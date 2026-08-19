//! `WinUIFilePathInputRenderer` — default `FilePathInputRenderer` impl.

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, CursorStyle, Div, Hsla, InteractiveElement, MouseButton,
    ParentElement, Pixels, SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div,
    px,
};

use yororen_ui_core::headless::file_path_input::FilePathInputProps;
use yororen_ui_core::headless::icon::{IconSource, icon};
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::headless::text_input_element::{
    TextInputElement, start_cursor_blink, wire_input_keyboard,
};
use yororen_ui_core::renderer::file_path_input::{FilePathInputRenderState, FilePathInputRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::animation::{
    animated_input_border, set_interaction_hovered, set_interaction_pressed,
};
use crate::themes::default_font;

pub struct WinUIFilePathInputRenderer;

// Inherent helpers — *not* part of the `FilePathInputRenderer`
// trait surface.
impl WinUIFilePathInputRenderer {
    pub fn bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn focus_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn hover_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.ctrl_strong_stroke")
            .or_else(|| theme.get_color("border.muted"))
            .unwrap_or_default()
    }
    pub fn active_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn button_bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn button_fg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn button_hover_bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.file_path_input.min_height")
            .unwrap_or(0.0) as f32)
    }
    pub fn padding(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.file_path_input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    pub fn action_gap(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.file_path_input.action_gap")
            .unwrap_or(0.0) as f32)
    }
    pub fn border_radius(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn icon_size(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.file_path_input.icon_size")
            .unwrap_or(0.0) as f32)
    }
}

impl FilePathInputRenderer for WinUIFilePathInputRenderer {
    fn compose(&self, props: &FilePathInputProps, cx: &mut App, window: &mut Window) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

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
        let render_state = FilePathInputRenderState {
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
        let button_fg = self.button_fg(&render_state, &theme);
        let button_bg = self.button_bg(&render_state, &theme);
        let min_h = self.min_height(&render_state, &theme);
        let padding = self.padding(&render_state, &theme);
        let radius = self.border_radius(&render_state, &theme);
        let action_gap = self.action_gap(&render_state, &theme);
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
            .gap(action_gap)
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

        let on_browse_clone = on_browse.clone();
        let window_handle = window.window_handle();
        let on_change_for_async = state.read(cx).on_change.clone();
        let state_for_browse = state.clone();

        let assembled: Stateful<Div> = keyed.child(
                icon(
                    "file-path-input-leading-icon",
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
                    .id("file-path-input-browse")
                    .size(icon_size)
                    .bg(button_bg)
                    .rounded(px(4.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(button_fg)
                    .cursor(CursorStyle::PointingHand)
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
                            "file-path-input-browse-icon",
                            IconSource::Builtin("file".into()),
                            cx,
                        )
                        .size(icon_size)
                        .color(button_fg)
                        .render(cx),
                    ),
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

pub fn arc_file_path_input<T: FilePathInputRenderer + 'static>(
    r: T,
) -> Arc<dyn FilePathInputRenderer> {
    Arc::new(r)
}
