//! `ToggleButtonRenderer` — visual side of `ToggleButton`.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, ElementId, FocusHandle, Hsla, InteractiveElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::headless::toggle_button::ToggleButtonProps;
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;

use yororen_ui_core::renderer::variant::VariantState;

use crate::animation::{
    AnimatedStateElement, control_config, lerp_hsla, set_interaction_hovered,
    set_interaction_pressed,
};
use crate::renderers::button::WinUIButtonRenderer;
use crate::themes::default_font;

pub use yororen_ui_core::renderer::toggle_button::{ToggleButtonRenderState, ToggleButtonRenderer};

pub struct WinUIToggleButtonRenderer;

// Inherent helpers — *not* part of the `ToggleButtonRenderer`
// trait surface. They exist so `compose` can stay readable and
// so unit tests can assert on individual palette paths.
impl WinUIToggleButtonRenderer {
    pub fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            // ToggleButton has a binary visual state (selected vs. not);
            // the registered custom variant controls the unselected
            // look. When selected we keep mapping to theme.primary so
            // existing toggle semantics are preserved.
            if state.selected {
                return theme.get_color("action.primary.bg").unwrap_or_default();
            }
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.disabled {
            // WinUI: a checked-but-disabled toggle uses the accent
            // disabled fill, not the neutral one.
            if state.selected {
                return theme
                    .get_color("action.primary.disabled_bg")
                    .unwrap_or_default();
            }
            theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or_default()
        } else if state.selected {
            theme.get_color("action.primary.bg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.bg").unwrap_or_default()
        }
    }
    pub fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            if state.selected {
                return theme.get_color("action.primary.fg").unwrap_or_default();
            }
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_fg")
                .unwrap_or_default();
        }
        if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.fg").unwrap_or_default()
        }
    }
    pub fn hover_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return self.bg(state, theme);
        }
        if state.selected {
            return theme
                .get_color("action.primary.hover_bg")
                .unwrap_or_default();
        }
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    pub fn active_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return self.bg(state, theme);
        }
        if state.selected {
            return theme
                .get_color("action.primary.active_bg")
                .unwrap_or_default();
        }
        theme
            .get_color("action.neutral.active_bg")
            .unwrap_or_default()
    }
    pub fn min_height(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.toggle_button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn disabled_opacity(&self, state: &ToggleButtonRenderState, _theme: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        1.0
    }
}

impl ToggleButtonRenderer for WinUIToggleButtonRenderer {
    fn compose(
        &self,
        props: &ToggleButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = ToggleButtonRenderState {
            variant: props.variant,
            selected: props.selected,
            disabled: props.disabled,
            custom_style: None,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let min_h = self.min_height(&state, theme);
        let radius = self.border_radius(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let icon_gap = theme
            .get_number("tokens.control.toggle_button.icon_gap")
            .unwrap_or(8.0) as f32;
        let horizontal_padding = theme
            .get_number("tokens.control.toggle_button.horizontal_padding")
            .unwrap_or(11.0) as f32;
        let font_size = px(theme
            .get_number("tokens.typography.font_size_md")
            .unwrap_or(14.0) as f32);

        // WinUI elevation border (same pair as the plain button):
        // checked toggles are accent-styled, unchecked ones use the
        // neutral pair — mirroring Accent/Default button styles.
        let button_state = yororen_ui_core::renderer::button::ButtonRenderState {
            variant: if state.selected {
                yororen_ui_core::renderer::variant::ActionVariantKind::Primary
            } else {
                yororen_ui_core::renderer::variant::ActionVariantKind::Neutral
            },
            disabled: props.disabled,
            ..Default::default()
        };
        let border = WinUIButtonRenderer
            .border(&button_state, theme)
            .unwrap_or_else(|| {
                yororen_ui_core::renderer::spec::BorderSpec::new(
                    px(1.0),
                    theme.get_color("border.default").unwrap_or_default(),
                )
            });
        let border_top = WinUIButtonRenderer.border_top(&button_state, theme);

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .relative()
            .font_family(default_font(theme))
            .text_size(font_size)
            .line_height(px(20.0))
            .text_color(fg)
            .min_h(min_h)
            .rounded(radius)
            .border_1()
            .border_color(border.color)
            .px(px(horizontal_padding))
            .py(px(0.))
            .gap(px(icon_gap))
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        let config = control_config(theme);
        let fill = AnimatedStateElement::new(
            (props.id.clone(), "fill"),
            props.id.clone(),
            props.selected,
            div().absolute().inset_0().rounded(radius),
            config,
            move |d: Div, hover, pressed, _checked| {
                let mut next = lerp_hsla(bg, hover_bg, hover);
                if pressed > 0.0 {
                    next = lerp_hsla(next, active_bg, pressed);
                }
                d.bg(next)
            },
        );
        el = el.child(fill);

        if !props.disabled {
            el = el.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(1.0))
                    .rounded_t(radius)
                    .bg(border_top),
            );
        }

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_size = props.icon_size;
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(icon_size),
                color: Some(fg),
            }
            .render(cx);
            el = el.child(icon_el);
        }
        if let Some(caption) = props.caption.clone() {
            el = el.child(caption);
        }

        if props.disabled {
            el = el.cursor(CursorStyle::OperationNotAllowed);
        } else {
            el = el
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

        el
    }
}

pub fn arc_toggle_button<T: ToggleButtonRenderer + 'static>(r: T) -> Arc<dyn ToggleButtonRenderer> {
    Arc::new(r)
}
