//! `TokenCardRenderer` — default `CardRenderer` impl.

use std::sync::Arc;

use gpui::{App, CursorStyle, Div, Hsla, Pixels, Styled, div};

use yororen_ui_core::headless::card::CardProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::card::{CardRenderState, CardRenderer};

pub struct TokenCardRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenCardRenderer {
    pub fn bg(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        // Use `surface.raised` rather than `surface.base` so the card
        // is visually distinct from the window background (which is
        // usually `surface.base`).
        theme.get_color("surface.raised").unwrap_or_default()
    }
    pub fn border(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn padding(&self, _state: &CardRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32,
        ))
    }
    pub fn border_radius(&self, _state: &CardRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(0.0) as f32)
    }
    pub fn shadow_alpha(&self, _state: &CardRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_1").unwrap_or_default().alpha
    }
    pub fn gap(&self, _state: &CardRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0) as f32)
    }
}

impl CardRenderer for TokenCardRenderer {
    fn compose(&self, props: &CardProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = CardRenderState {
            has_custom_bg: props.has_custom_bg,
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let pad = self.padding(&state, theme);
        let r = self.border_radius(&state, theme);
        let gap = self.gap(&state, theme);
        div()
            .flex()
            .flex_col()
            .gap(gap)
            .bg(bg)
            .border_1()
            .border_color(border)
            .p(pad.top)
            .rounded(r)
            .cursor(if props.interactive {
                CursorStyle::PointingHand
            } else {
                CursorStyle::Arrow
            })
    }
}

pub fn arc_card<T: CardRenderer + 'static>(r: T) -> Arc<dyn CardRenderer> {
    Arc::new(r)
}
