//! `LabelRenderer` — the visual side of `Label`.

use std::sync::Arc;

use gpui::{App, Div, FontWeight, Hsla, ParentElement, SharedString, Styled};

use yororen_ui_core::headless::label::LabelProps;
use yororen_ui_core::theme::Theme;

use crate::themes::default_font;

pub use yororen_ui_core::renderer::label::{LabelRenderState, LabelRenderer};

pub struct WinUILabelRenderer;

// Inherent helpers — *not* part of the `LabelRenderer` trait
// surface.
impl WinUILabelRenderer {
    pub fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla {
        if state.inherit_color {
            theme.get_color("content.primary").unwrap_or_default()
        } else if state.muted {
            theme.get_color("content.secondary").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    pub fn strong_weight(&self, _state: &LabelRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_semibold")
                .unwrap_or(600.0) as f32,
        )
    }
    pub fn family_mono(&self, _state: &LabelRenderState, theme: &Theme) -> SharedString {
        theme
            .get_string("tokens.typography.family_mono")
            .unwrap_or("ui-monospace")
            .to_string()
            .into()
    }
}

impl LabelRenderer for WinUILabelRenderer {
    fn compose(&self, props: &LabelProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = LabelRenderState {
            muted: props.muted,
            strong: props.strong,
            mono: props.mono,
            inherit_color: props.inherit_color,
            ellipsis: props.ellipsis,
            wrap: props.wrap,
            max_lines: props.max_lines,
        };
        let color = self.color(&state, theme);
        let weight = self.strong_weight(&state, theme);
        let family = self.family_mono(&state, theme);
        let mut el: Div = gpui::div();
        if !props.inherit_color {
            el = el.text_color(color);
        }
        if props.mono {
            el = el.font_family(family);
        } else {
            el = el.font_family(default_font(theme));
        }
        if props.strong {
            el = el.font_weight(weight);
        }
        if props.ellipsis {
            el = el.overflow_hidden().text_ellipsis().whitespace_nowrap();
        }
        if props.wrap {
            el = el.whitespace_normal();
        }
        if let Some(n) = props.max_lines {
            el = el.line_clamp(n).overflow_hidden();
        }
        el = el.child(props.text.clone());
        el
    }
}

pub fn arc_label<T: LabelRenderer + 'static>(r: T) -> Arc<dyn LabelRenderer> {
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
    fn color_picks_primary_when_not_muted() {
        let theme = fixture();
        let r = WinUILabelRenderer;
        let state = LabelRenderState::default();
        assert_eq!(
            r.color(&state, &theme),
            theme.get_color("content.primary").unwrap(),
        );
    }

    #[test]
    fn color_picks_secondary_when_muted() {
        let theme = fixture();
        let r = WinUILabelRenderer;
        let state = LabelRenderState {
            muted: true,
            ..Default::default()
        };
        assert_eq!(
            r.color(&state, &theme),
            theme.get_color("content.secondary").unwrap(),
        );
    }

    #[test]
    fn color_returns_primary_when_inherit() {
        let theme = fixture();
        let r = WinUILabelRenderer;
        let state = LabelRenderState {
            inherit_color: true,
            ..Default::default()
        };
        assert_eq!(
            r.color(&state, &theme),
            theme.get_color("content.primary").unwrap(),
        );
    }

    #[test]
    fn strong_weight_reads_weight_semibold() {
        let theme = fixture();
        let r = WinUILabelRenderer;
        let state = LabelRenderState::default();
        let expected = theme
            .get_number("tokens.typography.weight_semibold")
            .unwrap_or(600.0) as f32;
        assert_eq!(r.strong_weight(&state, &theme), FontWeight(expected));
    }

    #[test]
    fn family_mono_reads_family_mono() {
        let theme = fixture();
        let r = WinUILabelRenderer;
        let state = LabelRenderState::default();
        assert_eq!(
            r.family_mono(&state, &theme).to_string(),
            theme
                .get_string("tokens.typography.family_mono")
                .unwrap_or("ui-monospace"),
        );
    }
}
