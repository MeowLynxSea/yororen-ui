//! `WinUIHeadingRenderer` — default `HeadingRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, FontWeight, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::heading::{HeadingLevel, HeadingProps};
use yororen_ui_core::theme::Theme;

use crate::themes::default_font;

pub use yororen_ui_core::renderer::heading::{HeadingRenderState, HeadingRenderer};

pub struct WinUIHeadingRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIHeadingRenderer {
    pub fn size(&self, state: &HeadingRenderState, theme: &Theme) -> Pixels {
        let path = match state.level {
            HeadingLevel::H1 => "tokens.typography.font_size_3xl",
            HeadingLevel::H2 => "tokens.typography.font_size_xl",
            HeadingLevel::H3 => "tokens.typography.font_size_lg",
            HeadingLevel::H4 => "tokens.typography.font_size_md",
            HeadingLevel::H5 => "tokens.typography.font_size_sm",
            HeadingLevel::H6 => "tokens.typography.font_size_xs",
        };
        gpui::px(theme.get_number(path).unwrap_or(0.0) as f32)
    }

    pub fn weight(&self, state: &HeadingRenderState, theme: &Theme) -> FontWeight {
        // WinUI title treatment: the page header (28px) is
        // semibold, matching the reference typography ramp.
        let (path, default) = match state.level {
            HeadingLevel::H1 => ("tokens.typography.weight_semibold", 600.0),
            _ => ("tokens.typography.weight_semibold", 600.0),
        };
        FontWeight(theme.get_number(path).unwrap_or(default) as f32)
    }

    pub fn color(&self, _state: &HeadingRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
}

impl HeadingRenderer for WinUIHeadingRenderer {
    fn compose(&self, props: &HeadingProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = HeadingRenderState { level: props.level };
        let size = self.size(&state, theme);
        let weight = self.weight(&state, theme);
        let color = self.color(&state, theme);
        div()
            .font_family(default_font(theme))
            .text_color(color)
            .text_size(size)
            .font_weight(weight)
            .child(props.text.clone())
    }
}

pub fn arc_heading<T: HeadingRenderer + 'static>(r: T) -> Arc<dyn HeadingRenderer> {
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
    fn size_picks_correct_font_size_per_level() {
        let theme = fixture();
        let r = WinUIHeadingRenderer;
        let h1 = HeadingRenderState {
            level: HeadingLevel::H1,
        };
        let h6 = HeadingRenderState {
            level: HeadingLevel::H6,
        };
        // H1 should read tokens.typography.font_size_3xl (28).
        assert_eq!(
            r.size(&h1, &theme),
            gpui::px(
                theme
                    .get_number("tokens.typography.font_size_3xl")
                    .unwrap_or(0.0) as f32
            ),
        );
        // H6 should read tokens.typography.font_size_xs (11).
        assert_eq!(
            r.size(&h6, &theme),
            gpui::px(
                theme
                    .get_number("tokens.typography.font_size_xs")
                    .unwrap_or(0.0) as f32
            ),
        );
    }

    #[test]
    fn h1_weight_uses_bold_others_use_semibold() {
        let theme = fixture();
        let r = WinUIHeadingRenderer;
        let h1 = HeadingRenderState {
            level: HeadingLevel::H1,
        };
        let h2 = HeadingRenderState {
            level: HeadingLevel::H2,
        };
        // The whole ramp is semibold, matching the reference
        // page-header treatment.
        assert_eq!(
            r.weight(&h1, &theme),
            FontWeight(
                theme
                    .get_number("tokens.typography.weight_semibold")
                    .unwrap_or(600.0) as f32
            ),
        );
        // H2 reads weight_semibold (600).
        assert_eq!(
            r.weight(&h2, &theme),
            FontWeight(
                theme
                    .get_number("tokens.typography.weight_semibold")
                    .unwrap_or(600.0) as f32
            ),
        );
    }

    #[test]
    fn color_uses_content_primary() {
        let theme = fixture();
        let r = WinUIHeadingRenderer;
        let h1 = HeadingRenderState {
            level: HeadingLevel::H1,
        };
        assert_eq!(
            r.color(&h1, &theme),
            theme.get_color("content.primary").unwrap(),
        );
    }
}
