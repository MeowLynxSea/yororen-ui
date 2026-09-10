//! `TokenTextRenderer` — default `TextRenderer` impl.
//!
//! Builds a `Stateful<Div>` carrying the element id, the text string,
//! and the theme-derived size / color.

use std::sync::Arc;

use gpui::{App, Div, Hsla, InteractiveElement, ParentElement, Pixels, Stateful, Styled, div};

use yororen_ui_core::headless::text::TextProps;
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::text::{TextRenderState, TextRenderer};

pub struct TokenTextRenderer;

impl TokenTextRenderer {
    pub fn size(&self, state: &TextRenderState, theme: &Theme) -> Pixels {
        if state.has_custom_size {
            return gpui::px(0.0);
        }
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_md")
                .unwrap_or(14.0) as f32,
        )
    }

    pub fn color(&self, state: &TextRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_color {
            return gpui::rgb_to_hsla(gpui::rgb(0x0A0A0A));
        }
        theme
            .get_color("content.primary")
            .unwrap_or_else(|| gpui::rgb_to_hsla(gpui::rgb(0x0A0A0A)))
    }
}

impl TextRenderer for TokenTextRenderer {
    fn compose(&self, props: &TextProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = TextRenderState {
            has_custom_size: props.size.is_some(),
            has_custom_color: false,
        };

        div()
            .id(props.id.clone())
            .text_size(props.size.unwrap_or_else(|| self.size(&state, theme)))
            .text_color(self.color(&state, theme))
            .child(props.text.clone())
    }
}

pub fn arc_text<T: TextRenderer + 'static>(r: T) -> Arc<dyn TextRenderer> {
    Arc::new(r)
}
