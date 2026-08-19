//! `WinUIButtonRenderer` — default `ButtonRenderer` impl.
//!
//! Trait surface is **just** `compose` (see
//! `yororen_ui_core::renderer::button`). The helper methods
//! below (`bg` / `fg` / `padding` / `min_height` / …) are
//! inherent — they exist so other token-style renderers in
//! this crate can reuse the same palette lookups without
//! reimplementing them, and so unit tests can assert on the
//! palette directly.
//!
//! `Theme` here is the v0.3 JSON-backed theme from
//! `yororen_ui_core::theme` — no fixed schema, just
//! dot-separated paths. `WinUIButtonRenderer` reads:
//!
//! - `action.{neutral,primary,danger}.{bg,fg,hover_bg,active_bg,disabled_bg,disabled_fg}` for colours
//! - `tokens.control.button.{min_height,horizontal_padding,vertical_padding,radius,icon_gap}` for geometry

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, ElementId, FocusHandle, Hsla, InteractiveElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::headless::button::ButtonProps;
use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::renderer::spec::{BorderSpec, Edges, ShadowSpec};
use yororen_ui_core::renderer::variant::VariantState;
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;

use crate::animation::{
    AnimatedStateElement, control_config, lerp_hsla, set_interaction_hovered,
    set_interaction_pressed,
};
use crate::themes::default_font;

pub use yororen_ui_core::renderer::button::{ButtonRenderState, ButtonRenderer};
pub use yororen_ui_core::renderer::variant::ActionVariantKind;

/// Default implementation. Reads colour from
/// `action.<variant>.<bg|fg|hover_bg|…>` and geometry from
/// `tokens.control.button.*`.
///
/// When `state.custom_style` is `Some`, the colour helpers
/// delegate to the registered `VariantStyle` (passing the
/// current `disabled` flag through `VariantState`).
/// Non-colour properties (padding, radius, height) continue to
/// come from the theme.
pub struct WinUIButtonRenderer;

// Inherent helpers — these are *not* part of the
// `ButtonRenderer` trait surface. They exist so other
// renderers can share the same palette lookups by depending on
// `WinUIButtonRenderer` directly, and so unit tests can assert
// on individual token paths.
impl WinUIButtonRenderer {
    pub fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        let key = format!("action.{}.{}", state.variant.as_str(), field);
        theme.get_color(&key).unwrap_or_default()
    }

    pub fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled { "disabled_fg" } else { "fg" };
        let key = format!("action.{}.{}", state.variant.as_str(), field);
        theme.get_color(&key).unwrap_or_default()
    }

    pub fn padding(&self, _state: &ButtonRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.button.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme
            .get_number("tokens.control.button.vertical_padding")
            .unwrap_or((h as f64) / 2.0) as f32;
        Edges::symmetric(px(h), px(v))
    }

    pub fn border_radius(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.radius")
            .or_else(|| theme.get_number("tokens.radii.md"))
            .unwrap_or(6.0) as f32)
    }

    /// WinUI's "elevation border": the base 1px stroke is the darker
    /// bottom family (`ctrl_border_accent` on neutral, the accent
    /// pair on primary/danger), with a lighter 1px overlay drawn on
    /// the top edge by `border_top` — approximating the reference's
    /// `linear-gradient(180deg, ctrl-border, ctrl-border-accent)`.
    pub fn border(&self, state: &ButtonRenderState, theme: &Theme) -> Option<BorderSpec> {
        let color = if state.variant == ActionVariantKind::Neutral {
            theme
                .get_color("winui.ctrl_border_accent")
                .or_else(|| theme.get_color("border.default"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.accent_border_accent")
                .or_else(|| theme.get_color("border.default"))
                .unwrap_or_default()
        };
        Some(BorderSpec::new(px(1.0), color))
    }

    /// The lighter top edge of the elevation border (see [`border`]).
    pub fn border_top(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if state.variant == ActionVariantKind::Neutral {
            theme
                .get_color("winui.ctrl_border")
                .or_else(|| theme.get_color("border.muted"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.accent_border")
                .or_else(|| theme.get_color("border.muted"))
                .unwrap_or_default()
        }
    }

    /// WinUI dims pressed button text: default buttons fall back to
    /// `--text-secondary`, accent buttons to `--accent-text-secondary`.
    pub fn pressed_fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if state.variant == ActionVariantKind::Neutral {
            theme
                .get_color("winui.text_secondary")
                .or_else(|| theme.get_color("content.secondary"))
                .unwrap_or_default()
        } else {
            theme
                .get_color("winui.accent_text_secondary")
                .or_else(|| theme.get_color("content.secondary"))
                .unwrap_or_default()
        }
    }

    /// WinUI body text on controls: 14px.
    pub fn font_size(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.typography.font_size_md")
            .unwrap_or(14.0) as f32)
    }

    /// WinUI buttons do not use a drop shadow by default; only
    /// flyouts and elevated surfaces do.
    pub fn shadow(&self, _state: &ButtonRenderState, _theme: &Theme) -> Option<ShadowSpec> {
        None
    }

    pub fn min_height(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.min_height")
            .unwrap_or(36.0) as f32)
    }

    pub fn disabled_opacity(&self, state: &ButtonRenderState, _theme: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        1.0
    }

    pub fn hover_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "hover_bg"
        };
        let key = format!("action.{}.{}", state.variant.as_str(), field);
        theme.get_color(&key).unwrap_or_default()
    }

    pub fn active_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "active_bg"
        };
        let key = format!("action.{}.{}", state.variant.as_str(), field);
        theme.get_color(&key).unwrap_or_default()
    }
}

impl ButtonRenderer for WinUIButtonRenderer {
    fn compose(&self, props: &ButtonProps, focus_handle: &FocusHandle, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = ButtonRenderState {
            variant: props.variant,
            disabled: props.disabled,
            ..Default::default()
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let padding = self.padding(&state, theme);
        let radius = self.border_radius(&state, theme);
        let min_h = self.min_height(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let pressed_fg = self.pressed_fg(&state, theme);
        let border = self.border(&state, theme);
        let border_top = self.border_top(&state, theme);
        let font_size = self.font_size(&state, theme);
        let icon_gap = theme
            .get_number("tokens.control.button.icon_gap")
            .unwrap_or(8.0) as f32;

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .relative()
            .font_family(default_font(theme))
            .text_size(font_size)
            .line_height(px(20.0))
            .text_color(fg)
            .min_h(min_h)
            .rounded(radius)
            .px(padding.left)
            .py(padding.top)
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(border) = border {
            el = el.border_1().border_color(border.color);
        }

        // WinUI state transition: 167ms `cubic-bezier(0, 0, 0, 1)`.
        let config = control_config(theme);

        // Animated fill layer (rest → hover → pressed).
        let fill = AnimatedStateElement::new(
            (props.id.clone(), "fill"),
            props.id.clone(),
            false,
            div().absolute().inset_0().rounded(radius),
            config.clone(),
            move |d: Div, hover, pressed, _checked| {
                let mut next = lerp_hsla(bg, hover_bg, hover);
                if pressed > 0.0 {
                    next = lerp_hsla(next, active_bg, pressed);
                }
                d.bg(next)
            },
        );
        el = el.child(fill);

        // Elevation border: lighter 1px line across the top edge.
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

        // Content (icon + caption) with pressed text dimming.
        let mut content = div().flex().items_center().gap(px(icon_gap));
        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(props.icon_size),
                color: Some(fg),
            }
            .render(cx);
            content = content.child(icon_el);
        }
        if let Some(caption) = props.caption.clone() {
            content = content.child(caption);
        }
        let content = AnimatedStateElement::new(
            (props.id.clone(), "fg"),
            props.id.clone(),
            false,
            content,
            config,
            move |d: Div, _hover, pressed, _checked| {
                if pressed > 0.0 {
                    d.text_color(lerp_hsla(fg, pressed_fg, pressed))
                } else {
                    d.text_color(fg)
                }
            },
        );
        el = el.child(content);

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

/// Convenience: build a registry entry that wraps the given
/// renderer in an Arc.
pub fn arc<T: ButtonRenderer + 'static>(r: T) -> Arc<dyn ButtonRenderer> {
    Arc::new(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yororen_ui_core::theme::Theme;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/winui-dark.json");
        Theme::from_json(json).expect("winui-dark.json is valid")
    }

    #[test]
    fn token_button_renderer_returns_primary_palette() {
        let theme = fixture();
        let r = WinUIButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            disabled: false,
            ..Default::default()
        };
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.primary.bg").unwrap()
        );
        assert_eq!(
            r.fg(&state, &theme),
            theme.get_color("action.primary.fg").unwrap()
        );
    }

    #[test]
    fn disabled_uses_disabled_palette() {
        let theme = fixture();
        let r = WinUIButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            disabled: true,
            ..Default::default()
        };
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.primary.disabled_bg").unwrap()
        );
        assert_eq!(
            r.fg(&state, &theme),
            theme.get_color("action.primary.disabled_fg").unwrap()
        );
    }

    #[test]
    fn neutral_variant_picks_neutral_palette() {
        let theme = fixture();
        let r = WinUIButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Neutral,
            ..Default::default()
        };
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.neutral.bg").unwrap()
        );
    }

    #[test]
    fn danger_variant_picks_danger_palette() {
        let theme = fixture();
        let r = WinUIButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Danger,
            ..Default::default()
        };
        assert_eq!(
            r.bg(&state, &theme),
            theme.get_color("action.danger.bg").unwrap()
        );
    }

    #[test]
    fn min_height_uses_control_button_token() {
        let theme = fixture();
        let r = WinUIButtonRenderer;
        let state = ButtonRenderState::default();
        let expected = theme
            .get_number("tokens.control.button.min_height")
            .unwrap() as f32;
        assert_eq!(r.min_height(&state, &theme), gpui::px(expected));
    }

    #[test]
    fn hover_and_active_bg_read_action_hover_and_active_paths() {
        let theme = fixture();
        let r = WinUIButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        assert_eq!(
            r.hover_bg(&state, &theme),
            theme.get_color("action.primary.hover_bg").unwrap(),
        );
        assert_eq!(
            r.active_bg(&state, &theme),
            theme.get_color("action.primary.active_bg").unwrap(),
        );
        assert_ne!(r.bg(&state, &theme), r.hover_bg(&state, &theme));
        assert_ne!(r.hover_bg(&state, &theme), r.active_bg(&state, &theme));
    }

    #[test]
    fn missing_path_yields_zero_color_doesnt_panic() {
        let theme = Theme::from_value(serde_json::json!({}));
        let r = WinUIButtonRenderer;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        let _ = r.bg(&state, &theme);
        let _ = r.fg(&state, &theme);
        let _ = r.padding(&state, &theme);
        let _ = r.border_radius(&state, &theme);
        let _ = r.min_height(&state, &theme);
    }
}
