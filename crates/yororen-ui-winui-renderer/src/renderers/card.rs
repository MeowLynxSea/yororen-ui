//! `WinUICardRenderer` — default `CardRenderer` impl.

use std::sync::Arc;

use gpui::{App, BoxShadow, CursorStyle, Div, Hsla, InteractiveElement, Pixels, Styled, div};

use yororen_ui_core::headless::card::CardProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::card::{CardRenderState, CardRenderer};

pub struct WinUICardRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUICardRenderer {
    pub fn bg(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        // WinUI cards are translucent surface fills with a hairline
        // border, not heavy solid cards.
        theme
            .get_color("winui.card_bg")
            .or_else(|| theme.get_color("surface.raised"))
            .unwrap_or_default()
    }
    pub fn border(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.card_stroke")
            .or_else(|| theme.get_color("border.default"))
            .unwrap_or_default()
    }
    pub fn padding(&self, _state: &CardRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme
                .get_number("tokens.control.card.padding")
                .or_else(|| theme.get_number("tokens.spacing.inset_md"))
                .unwrap_or(24.0) as f32,
        ))
    }
    pub fn border_radius(&self, _state: &CardRenderState, theme: &Theme) -> Pixels {
        // WinUI SettingsCard uses the 4px ControlCornerRadius (not
        // the 8px overlay radius).
        gpui::px(
            theme
                .get_number("tokens.control.card.radius")
                .or_else(|| theme.get_number("tokens.radii.sm"))
                .unwrap_or(4.0) as f32,
        )
    }
    pub fn hover_bg(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.ctrl_fill_hover")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default()
    }
    pub fn shadow(&self, _state: &CardRenderState, _theme: &Theme) -> Option<BoxShadow> {
        // The reference SettingsCard is flat — hairline stroke only.
        None
    }
    pub fn gap(&self, _state: &CardRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0) as f32)
    }
}

impl CardRenderer for WinUICardRenderer {
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
        let shadow = self.shadow(&state, theme);
        let hover_bg = self.hover_bg(&state, theme);

        let hover_stroke = theme
            .get_color("winui.ctrl_border_accent")
            .or_else(|| theme.get_color("border.default"))
            .unwrap_or_default();

        // NOTE: the headless layer stamps `.id(props.id)` onto the
        // returned container and callers append content to that SAME
        // element — so compose must return the content container
        // itself (single flex column), never an outer wrapper.
        // Interactive hover therefore uses the instant `.hover()`
        // refinement (the reference's 83ms fill transition is below
        // perception at this duration).
        let mut el = div()
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
            });
        if let Some(shadow) = shadow {
            el = el.shadow(vec![shadow]);
        }
        if props.interactive {
            // Hover: fill lightens AND the stroke deepens, per the
            // reference SettingsCard "clickable" state.
            el = el.hover(move |s| {
                s.bg(hover_bg).border_color(gpui::Hsla {
                    a: 1.0,
                    ..hover_stroke
                })
            });
        }
        el
    }
}

pub fn arc_card<T: CardRenderer + 'static>(r: T) -> Arc<dyn CardRenderer> {
    Arc::new(r)
}
