//! `WinUIRadioRenderer` — default `RadioRenderer` impl.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, CursorStyle, Div, FocusHandle, Hsla, InteractiveElement, MouseButton, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::animation::AnimationConfig;
use yororen_ui_core::headless::radio::RadioProps;
use yororen_ui_core::theme::Theme;

use crate::animation::{
    AnimatedStateElement, lerp_f32, lerp_hsla, set_interaction_hovered, set_interaction_pressed,
};

pub use yororen_ui_core::renderer::radio::{RadioRenderState, RadioRenderer};

pub struct WinUIRadioRenderer;

// Inherent helpers — *not* part of the `RadioRenderer` trait
// surface.
impl WinUIRadioRenderer {
    pub fn ring_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.radio.ring_size")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn dot_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.radio.dot_size")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("winui.ctrl_fill_disabled")
                .or_else(|| theme.get_color("surface.sunken"))
                .unwrap_or_default()
        } else if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or_default()
            } else {
                theme
                    .get_color("winui.accent")
                    .or_else(|| theme.get_color("action.primary.bg"))
                    .unwrap_or_default()
            }
        } else {
            theme
                .get_color("winui.ctrl_fill")
                .or_else(|| theme.get_color("surface.base"))
                .unwrap_or_default()
        }
    }
    pub fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or_default()
            } else {
                theme
                    .get_color("winui.accent")
                    .or_else(|| theme.get_color("action.primary.bg"))
                    .unwrap_or_default()
            }
        } else {
            theme
                .get_color("winui.ctrl_strong_stroke")
                .or_else(|| theme.get_color("border.default"))
                .unwrap_or_default()
        }
    }
    pub fn ring_hover_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            self.ring_bg(state, theme)
        } else if state.checked {
            theme
                .get_color("winui.accent_hover")
                .or_else(|| theme.get_color("action.primary.hover_bg"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.ctrl_fill_hover")
                .or_else(|| theme.get_color("surface.hover"))
                .unwrap_or_default()
        }
    }
    pub fn ring_active_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            self.ring_bg(state, theme)
        } else if state.checked {
            theme
                .get_color("winui.accent_pressed")
                .or_else(|| theme.get_color("action.primary.active_bg"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.ctrl_fill_pressed")
                .or_else(|| theme.get_color("surface.sunken"))
                .unwrap_or_default()
        }
    }
    pub fn ring_border_hover(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            self.ring_border(state, theme)
        } else if state.checked {
            theme
                .get_color("winui.accent_hover")
                .or_else(|| theme.get_color("action.primary.hover_bg"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("content.primary")
                .or_else(|| theme.get_color("border.strong"))
                .unwrap_or_default()
        }
    }
    pub fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("winui.ctrl_strong_stroke_disabled")
                .or_else(|| theme.get_color("content.disabled"))
                .unwrap_or_default()
        } else if state.has_custom_tone {
            state.custom_tone.unwrap_or_default()
        } else {
            theme
                .get_color("winui.accent_text")
                .or_else(|| theme.get_color("action.primary.fg"))
                .unwrap_or_default()
        }
    }
    pub fn focus_color(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn disabled_opacity(&self, _state: &RadioRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

impl RadioRenderer for WinUIRadioRenderer {
    fn compose(&self, props: &RadioProps, focus_handle: &FocusHandle, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = RadioRenderState {
            checked: props.checked,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let bg = self.ring_bg(&state, theme);
        let border = self.ring_border(&state, theme);
        let ring_size = self.ring_size(&state, theme);
        let dot_size = self.dot_size(&state, theme);
        let dot_fg = self.dot_fg(&state, theme);
        let pill_radius = px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32);
        let hover_bg = self.ring_hover_bg(&state, theme);
        let active_bg = self.ring_active_bg(&state, theme);
        let hover_border = self.ring_border_hover(&state, theme);

        let config = AnimationConfig::default().with_duration(Duration::from_millis(150));
        let dot_size_f: f32 = dot_size.into();

        // Inner dot: always mounted; `checked` scales it 0 → 1, hover
        // scales it up slightly and press squashes it (WinUI).
        let dot = AnimatedStateElement::new(
            (props.id.clone(), "radio-dot"),
            props.id.clone(),
            props.checked,
            div(),
            config.clone(),
            move |d: Div, hover, pressed, checked| {
                let size =
                    dot_size_f * checked * lerp_f32(1.0, lerp_f32(1.15, 0.8, pressed), hover);
                d.w(px(size))
                    .h(px(size))
                    .rounded(pill_radius)
                    .bg(dot_fg)
                    .opacity(checked)
            },
        );

        // Ring: animated fill + stroke between rest / hover / pressed.
        let ring = AnimatedStateElement::new(
            (props.id.clone(), "radio-ring"),
            props.id.clone(),
            false,
            div()
                .border_1()
                .size(ring_size)
                .rounded(pill_radius)
                .flex()
                .items_center()
                .justify_center()
                .child(dot),
            config,
            move |d: Div, hover, pressed, _checked| {
                let mut next_bg = lerp_hsla(bg, hover_bg, hover);
                if pressed > 0.0 {
                    next_bg = lerp_hsla(next_bg, active_bg, pressed);
                }
                let mut next_border = lerp_hsla(border, hover_border, hover);
                if pressed > 0.0 {
                    next_border = lerp_hsla(next_border, active_bg, pressed);
                }
                d.bg(next_bg).border_color(next_border)
            },
        );

        let mut shell: Stateful<Div> = div()
            .id(props.id.clone())
            .size(ring_size)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if props.disabled {
            shell = shell.cursor(CursorStyle::OperationNotAllowed);
        } else {
            shell = shell
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
                })
                .cursor(CursorStyle::PointingHand);
        }

        shell.child(ring)
    }
}

pub fn arc_radio<T: RadioRenderer + 'static>(r: T) -> Arc<dyn RadioRenderer> {
    Arc::new(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::rgb;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/winui-light.json");
        Theme::from_json(json).expect("winui-light.json is valid")
    }

    #[test]
    fn custom_tone_overrides_checked_ring_and_dot() {
        let theme = fixture();
        let r = WinUIRadioRenderer;
        let custom = rgb(0x123456).into();
        let state = RadioRenderState {
            checked: true,
            disabled: false,
            has_custom_tone: true,
            custom_tone: Some(custom),
        };
        assert_eq!(r.ring_border(&state, &theme), custom);
        assert_eq!(r.dot_fg(&state, &theme), custom);
    }
}
