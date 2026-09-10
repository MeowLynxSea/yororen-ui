//! `TokenCheckboxRenderer` — default `CheckboxRenderer` impl.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::headless::checkbox::CheckboxProps;
use yororen_ui_core::theme::Theme;

use crate::animation::AnimatedOpacityElement;

pub use yororen_ui_core::renderer::checkbox::{CheckboxRenderState, CheckboxRenderer};

pub struct TokenCheckboxRenderer;

// Inherent helpers — *not* part of the `CheckboxRenderer`
// trait surface.
impl TokenCheckboxRenderer {
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
            theme.get_color("surface.sunken").unwrap_or_default()
        } else if state.checked || state.indeterminate {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or_default()
            } else {
                theme.get_color("action.primary.bg").unwrap_or_default()
            }
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    pub fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked || state.indeterminate {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or_default()
            } else {
                theme.get_color("action.primary.bg").unwrap_or_default()
            }
        } else {
            theme.get_color("border.default").unwrap_or_default()
        }
    }
    pub fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked || state.indeterminate {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or_default()
        } else {
            theme.get_color("surface.hover").unwrap_or_default()
        }
    }
    pub fn box_active_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked || state.indeterminate {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or_default()
        } else {
            theme.get_color("surface.sunken").unwrap_or_default()
        }
    }
    pub fn check_fg(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.fg").unwrap_or_default()
    }
    pub fn focus_color(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn disabled_opacity(&self, _state: &CheckboxRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

impl CheckboxRenderer for TokenCheckboxRenderer {
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
            indeterminate: props.indeterminate,
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

        // The glyph (checkmark or mixed-state dash) is always
        // mounted and faded in/out so the state transition is
        // animated.
        let check_color = self.box_border(
            &CheckboxRenderState {
                checked: true,
                ..state
            },
            theme,
        );
        // The indeterminate dash is wider and shorter than the
        // checkmark square so the two states read apart at a
        // glance.
        let check_f: f32 = check_size.into();
        let glyph = if props.indeterminate {
            div()
                .bg(check_color)
                .w(px(check_f * 1.6))
                .h(px((check_f * 0.35).max(2.0)))
                .rounded(px(2.))
        } else {
            div().bg(check_color).size(check_size).rounded(px(2.))
        };
        let animated_check = AnimatedOpacityElement::new(
            (props.id.clone(), "check"),
            props.checked || props.indeterminate,
            glyph,
        );

        div()
            .id(props.id.clone())
            .bg(bg)
            .border_1()
            .border_color(border)
            .size(size)
            .rounded(px(4.))
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle)
            .child(animated_check)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
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
        let json = include_str!("../../themes/system-light.json");
        Theme::from_json(json).expect("system-light.json is valid")
    }

    #[test]
    fn custom_tone_overrides_checked_state_color() {
        let theme = fixture();
        let r = TokenCheckboxRenderer;
        let custom = gpui::rgb_to_hsla(rgb(0xabcdef));
        let state = CheckboxRenderState {
            checked: true,
            indeterminate: false,
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
