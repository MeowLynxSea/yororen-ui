//! `WinUIBadgeRenderer` — default `BadgeRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, FontWeight, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::badge::{BadgeProps, BadgeVariant};
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::badge::{BadgeRenderState, BadgeRenderer};

pub struct WinUIBadgeRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIBadgeRenderer {
    pub fn bg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        let key = match state.variant {
            BadgeVariant::Neutral => "neutral",
            BadgeVariant::Success => "success",
            BadgeVariant::Warning => "warning",
            BadgeVariant::Danger => "danger",
            BadgeVariant::Info => "info",
        };
        theme
            .get_color(&format!("status.{key}.bg"))
            .unwrap_or_default()
    }

    pub fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            theme.get_color("content.on_status").unwrap_or_default()
        } else {
            let key = match state.variant {
                BadgeVariant::Neutral => "neutral",
                BadgeVariant::Success => "success",
                BadgeVariant::Warning => "warning",
                BadgeVariant::Danger => "danger",
                BadgeVariant::Info => "info",
            };
            theme
                .get_color(&format!("status.{key}.fg"))
                .unwrap_or_default()
        }
    }

    pub fn padding_x(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32)
    }

    pub fn height(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.badge.min_height")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn font_size(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_xs")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn font_weight(&self, _state: &BadgeRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_medium")
                .unwrap_or(500.0) as f32,
        )
    }

    pub fn border_radius(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
    }
}

impl BadgeRenderer for WinUIBadgeRenderer {
    fn compose(&self, props: &BadgeProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = BadgeRenderState {
            variant: props.variant,
            has_custom_tone: false,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let px_v = self.padding_x(&state, theme);
        let h = self.height(&state, theme);
        let fs = self.font_size(&state, theme);
        let fw = self.font_weight(&state, theme);
        let r = self.border_radius(&state, theme);
        div()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .text_color(fg)
            .px(px_v)
            .h(h)
            .text_size(fs)
            .font_weight(fw)
            .rounded(r)
            .child(props.text.clone())
    }
}

pub fn arc_badge<T: BadgeRenderer + 'static>(r: T) -> Arc<dyn BadgeRenderer> {
    Arc::new(r)
}
