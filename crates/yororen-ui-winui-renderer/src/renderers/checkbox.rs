//! `WinUICheckboxRenderer` — default `CheckboxRenderer` impl.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, CursorStyle, Div, FocusHandle, Hsla, InteractiveElement, MouseButton, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::animation::AnimationConfig;
use yororen_ui_core::headless::checkbox::CheckboxProps;
use yororen_ui_core::theme::Theme;

use crate::animation::{
    AnimatedOpacityElement, AnimatedStateElement, lerp_hsla, set_interaction_hovered,
    set_interaction_pressed,
};

pub use yororen_ui_core::renderer::checkbox::{CheckboxRenderState, CheckboxRenderer};

pub struct WinUICheckboxRenderer;

// Inherent helpers — *not* part of the `CheckboxRenderer`
// trait surface.
impl WinUICheckboxRenderer {
    pub fn box_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.checkbox.box_size")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn check_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.checkbox.check_size")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
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
    pub fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
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
    pub fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
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
    pub fn box_active_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
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
    pub fn box_border_hover(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            self.box_border(state, theme)
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
    pub fn check_fg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("winui.text_disabled")
                .or_else(|| theme.get_color("content.disabled"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.accent_text")
                .or_else(|| theme.get_color("action.primary.fg"))
                .unwrap_or_default()
        }
    }
    pub fn focus_color(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn disabled_opacity(&self, _state: &CheckboxRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

impl CheckboxRenderer for WinUICheckboxRenderer {
    fn compose(
        &self,
        props: &CheckboxProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = CheckboxRenderState {
            checked: props.checked,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let bg = self.box_bg(&state, theme);
        let border = self.box_border(&state, theme);
        let size = self.box_size(&state, theme);
        let check_size = self.check_size(&state, theme);
        let hover_bg = self.box_hover_bg(&state, theme);
        let active_bg = self.box_active_bg(&state, theme);
        let hover_border = self.box_border_hover(&state, theme);

        // The checkmark is always mounted and faded in/out so the
        // checked state transition is animated.
        let check_color = self.check_fg(&state, theme);
        let check = div()
            .text_color(check_color)
            .text_size(check_size)
            .child("✓");
        let animated_check =
            AnimatedOpacityElement::new((props.id.clone(), "check"), props.checked, check);

        let config = AnimationConfig::default().with_duration(Duration::from_millis(150));

        // Box: animated fill + stroke between rest / hover / pressed.
        let box_animated = AnimatedStateElement::new(
            (props.id.clone(), "box"),
            props.id.clone(),
            false,
            div()
                .border_1()
                .size(size)
                .rounded(px(4.))
                .flex()
                .items_center()
                .justify_center()
                .child(animated_check),
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
            .size(size)
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

        shell.child(box_animated)
    }
}

pub fn arc_checkbox<T: CheckboxRenderer + 'static>(r: T) -> Arc<dyn CheckboxRenderer> {
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
    fn custom_tone_overrides_checked_state_color() {
        let theme = fixture();
        let r = WinUICheckboxRenderer;
        let custom = rgb(0xabcdef).into();
        let state = CheckboxRenderState {
            checked: true,
            disabled: false,
            has_custom_tone: true,
            custom_tone: Some(custom),
        };
        assert_eq!(r.box_bg(&state, &theme), custom);
        assert_eq!(r.box_border(&state, &theme), custom);
        let state_unchecked = CheckboxRenderState {
            checked: false,
            ..state
        };
        assert_ne!(r.box_bg(&state_unchecked, &theme), custom);
    }
}
