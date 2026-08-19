//! `WinUISwitchRenderer` — default `SwitchRenderer` impl.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, CursorStyle, Div, FocusHandle, Hsla, InteractiveElement, MouseButton, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::animation::AnimationConfig;
use yororen_ui_core::headless::switch::SwitchProps;
use yororen_ui_core::theme::Theme;

use crate::animation::{
    AnimatedMarginElement, AnimatedStateElement, lerp_f32, lerp_hsla, set_interaction_hovered,
    set_interaction_pressed,
};

pub use yororen_ui_core::renderer::switch::{SwitchRenderState, SwitchRenderer};

pub struct WinUISwitchRenderer;

// Inherent helpers — *not* part of the `SwitchRenderer` trait
// surface.
impl WinUISwitchRenderer {
    pub fn track_w(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.track_w")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn track_h(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.track_h")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn knob_size(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.knob_size")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn padding(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.padding")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
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
                .get_color("winui.subtle_fill_secondary")
                .or_else(|| theme.get_color("surface.hover"))
                .unwrap_or_default()
        }
    }
    pub fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("winui.ctrl_strong_stroke_disabled")
                .or_else(|| theme.get_color("border.muted"))
                .unwrap_or_default()
        } else if state.checked {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        } else {
            theme
                .get_color("winui.ctrl_strong_stroke")
                .or_else(|| theme.get_color("border.default"))
                .unwrap_or_default()
        }
    }
    pub fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("winui.accent_hover")
                .or_else(|| theme.get_color("action.primary.hover_bg"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.subtle_fill_tertiary")
                .or_else(|| theme.get_color("content.tertiary"))
                .unwrap_or_default()
        }
    }
    pub fn track_active_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("winui.accent_pressed")
                .or_else(|| theme.get_color("action.primary.active_bg"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.subtle_fill")
                .or_else(|| theme.get_color("surface.sunken"))
                .unwrap_or_default()
        }
    }
    pub fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("winui.ctrl_strong_stroke_disabled")
                .or_else(|| theme.get_color("content.disabled"))
                .unwrap_or_default()
        } else if state.checked {
            theme
                .get_color("winui.accent_text")
                .or_else(|| theme.get_color("action.primary.fg"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.ctrl_strong_stroke")
                .or_else(|| theme.get_color("content.primary"))
                .unwrap_or_default()
        }
    }
    /// Unchecked thumb colour while hovering / pressed. WinUI uses a
    /// stronger black/white than the resting `ctrl_strong_stroke`.
    pub fn knob_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("winui.ctrl_strong_stroke_disabled")
                .or_else(|| theme.get_color("content.disabled"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("content.primary")
                .or_else(|| theme.get_color("border.strong"))
                .unwrap_or_default()
        }
    }
    pub fn focus_color(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn disabled_opacity(&self, _state: &SwitchRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

impl SwitchRenderer for WinUISwitchRenderer {
    fn compose(&self, props: &SwitchProps, focus_handle: &FocusHandle, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = SwitchRenderState {
            checked: props.checked,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let track = self.track_bg(&state, theme);
        let w = self.track_w(&state, theme);
        let h = self.track_h(&state, theme);
        let knob_size = self.knob_size(&state, theme);
        let pad = self.padding(&state, theme);
        let pill_radius = px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32);
        let track_hover = self.track_hover_bg(&state, theme);
        let track_active = self.track_active_bg(&state, theme);
        let track_border = self.track_border(&state, theme);
        let track_border_hover = theme
            .get_color("content.primary")
            .or_else(|| theme.get_color("border.strong"))
            .unwrap_or(track_border);

        let unchecked_knob_color = self.knob_bg(
            &SwitchRenderState {
                checked: false,
                ..state
            },
            theme,
        );
        let unchecked_knob_hover_color = self.knob_hover_bg(
            &SwitchRenderState {
                checked: false,
                ..state
            },
            theme,
        );
        let checked_knob_color = self.knob_bg(
            &SwitchRenderState {
                checked: true,
                ..state
            },
            theme,
        );

        // Fast hover transition (~167ms in WinUI), faster still for
        // press feedback (~83ms).
        let fast_config = AnimationConfig::default().with_duration(Duration::from_millis(150));

        // The thumb: 12px at rest, 14px on hover, 17x14 while
        // pressed; colour interpolates between checked / unchecked and
        // the hover accent on the unchecked thumb.
        let knob_size_f: f32 = knob_size.into();
        let thumb = AnimatedStateElement::new(
            (props.id.clone(), "thumb"),
            props.id.clone(),
            props.checked,
            // Absolute so the thumb scales on BOTH axes and stays
            // centred inside the fixed 20x20 knob (flex centering can
            // be ambiguous for a child whose size animates).
            div().absolute(),
            fast_config.clone(),
            move |d: Div, hover, pressed, checked| {
                let mut tw = lerp_f32(12.0, 14.0, hover);
                let mut th = lerp_f32(12.0, 14.0, hover);
                tw = lerp_f32(tw, 17.0, pressed);
                th = lerp_f32(th, 14.0, pressed);
                let off_color = lerp_hsla(unchecked_knob_color, unchecked_knob_hover_color, hover);
                let color = lerp_hsla(off_color, checked_knob_color, checked);
                d.left(px((knob_size_f - tw) / 2.0))
                    .top(px((knob_size_f - th) / 2.0))
                    .w(px(tw))
                    .h(px(th))
                    .rounded(px(th / 2.0))
                    .bg(color)
            },
        );

        let knob_inner = div().relative().size(knob_size).child(thumb);

        let slide_distance = {
            let w_f: f32 = w.into();
            let knob_f: f32 = knob_size.into();
            let pad_f: f32 = pad.into();
            px((w_f - knob_f - pad_f * 2.0).max(0.0))
        };
        let knob_animated = AnimatedMarginElement::new(
            (props.id.clone(), "knob-slide"),
            props.checked,
            slide_distance,
            knob_inner,
        );

        // Track: animated background + border between
        // rest / hover / pressed (WinUI's `.track` transition).
        let track_animated = AnimatedStateElement::new(
            (props.id.clone(), "track"),
            props.id.clone(),
            false,
            div()
                .w(w)
                .h(h)
                .rounded(pill_radius)
                .border_1()
                .p(pad)
                .flex()
                .items_center()
                .justify_start()
                .child(knob_animated),
            fast_config,
            move |d: Div, hover, pressed, _checked| {
                let mut bg = lerp_hsla(track, track_hover, hover);
                if pressed > 0.0 {
                    bg = lerp_hsla(bg, track_active, pressed);
                }
                let border = lerp_hsla(track_border, track_border_hover, hover);
                d.bg(bg).border_color(border)
            },
        );

        let mut shell: Stateful<Div> = div()
            .id(props.id.clone())
            .relative()
            .w(w)
            .h(h)
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

        shell.child(track_animated)
    }
}

pub fn arc_switch<T: SwitchRenderer + 'static>(r: T) -> Arc<dyn SwitchRenderer> {
    Arc::new(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/winui-light.json");
        Theme::from_json(json).expect("winui-light.json is valid")
    }

    #[test]
    fn track_w_h_knob_size_padding_read_switch_tokens() {
        let theme = fixture();
        let r = WinUISwitchRenderer;
        let state = SwitchRenderState::default();
        assert_eq!(
            r.track_w(&state, &theme),
            gpui::px(
                theme
                    .get_number("tokens.control.switch.track_w")
                    .unwrap_or(0.0) as f32
            ),
        );
        assert_eq!(
            r.track_h(&state, &theme),
            gpui::px(
                theme
                    .get_number("tokens.control.switch.track_h")
                    .unwrap_or(0.0) as f32
            ),
        );
        assert_eq!(
            r.knob_size(&state, &theme),
            gpui::px(
                theme
                    .get_number("tokens.control.switch.knob_size")
                    .unwrap_or(0.0) as f32
            ),
        );
    }

    #[test]
    fn track_bg_uses_action_primary_when_checked() {
        let theme = fixture();
        let r = WinUISwitchRenderer;
        let state = SwitchRenderState {
            checked: true,
            ..Default::default()
        };
        assert_eq!(
            r.track_bg(&state, &theme),
            theme.get_color("action.primary.bg").unwrap(),
        );
    }

    #[test]
    fn track_bg_uses_surface_hover_when_unchecked() {
        let theme = fixture();
        let r = WinUISwitchRenderer;
        let state = SwitchRenderState {
            checked: false,
            ..Default::default()
        };
        assert_eq!(
            r.track_bg(&state, &theme),
            theme.get_color("surface.hover").unwrap(),
        );
    }

    #[test]
    fn disabled_state_doesnt_panic() {
        let theme = fixture();
        let r = WinUISwitchRenderer;
        let state = SwitchRenderState {
            disabled: true,
            ..Default::default()
        };
        let _ = r.track_bg(&state, &theme);
        let _ = r.knob_bg(&state, &theme);
        assert_eq!(r.disabled_opacity(&state, &theme), 0.5);
    }

    #[test]
    fn custom_tone_overrides_checked_track_color() {
        let theme = fixture();
        let r = WinUISwitchRenderer;
        let custom = gpui::rgb(0xdeadbe).into();
        let state = SwitchRenderState {
            checked: true,
            disabled: false,
            has_custom_tone: true,
            custom_tone: Some(custom),
        };
        assert_eq!(r.track_bg(&state, &theme), custom);
    }
}
