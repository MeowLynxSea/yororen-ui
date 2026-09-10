//! `TokenRadioRenderer` — default `RadioRenderer` impl.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::headless::radio::RadioProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::radio::{RadioRenderState, RadioRenderer};

pub struct TokenRadioRenderer;

// Inherent helpers — *not* part of the `RadioRenderer` trait
// surface.
impl TokenRadioRenderer {
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
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    pub fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or_default()
            } else {
                theme.get_color("action.primary.bg").unwrap_or_default()
            }
        } else {
            theme.get_color("border.default").unwrap_or_default()
        }
    }
    pub fn ring_hover_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn ring_active_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.sunken").unwrap_or_default()
    }
    pub fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            state.custom_tone.unwrap_or_default()
        } else {
            theme.get_color("action.primary.bg").unwrap_or_default()
        }
    }
    pub fn focus_color(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn disabled_opacity(&self, _state: &RadioRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

impl RadioRenderer for TokenRadioRenderer {
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

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_1()
            .border_color(border)
            .size(ring_size)
            .rounded(pill_radius)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);
        if props.checked {
            el = el.child(div().bg(dot_fg).size(dot_size).rounded(pill_radius));
        }
        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
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
        let json = include_str!("../../themes/system-light.json");
        Theme::from_json(json).expect("system-light.json is valid")
    }

    #[test]
    fn custom_tone_overrides_checked_ring_and_dot() {
        let theme = fixture();
        let r = TokenRadioRenderer;
        let custom = gpui::rgb_to_hsla(rgb(0x123456));
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
