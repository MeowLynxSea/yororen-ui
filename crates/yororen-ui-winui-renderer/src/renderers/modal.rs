//! `WinUIModalRenderer` — default `ModalRenderer` impl.
//!
//! Paints the modal *panel* (bg, border, padding, radius, shadow).
//! The caller is responsible for the scrim / overlay and for adding
//! children inside the panel after `.render(cx)`.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div, px};

use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::modal::ModalProps;
use yororen_ui_core::theme::Theme;

use crate::animation::AnimatedPresenceElement;

pub use yororen_ui_core::renderer::modal::{ModalRenderState, ModalRenderer};

pub struct WinUIModalRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIModalRenderer {
    pub fn panel_bg(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or_default()
    }
    pub fn panel_border(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    pub fn panel_padding(&self, _state: &ModalRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_lg").unwrap_or(16.0) as f32)
    }
    pub fn panel_border_radius(&self, _state: &ModalRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(8.0) as f32)
    }
    pub fn panel_shadow_alpha(&self, _state: &ModalRenderState, _theme: &Theme) -> f32 {
        0.15
    }
}

impl ModalRenderer for WinUIModalRenderer {
    fn compose(&self, props: &mut ModalProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ModalRenderState {};
        let panel_bg = self.panel_bg(&state, theme);
        let panel_border = self.panel_border(&state, theme);
        let pad = self.panel_padding(&state, theme);
        let r = self.panel_border_radius(&state, theme);
        let alpha = self.panel_shadow_alpha(&state, theme);

        let visible = props.state.read(cx).is_visible();
        if !visible {
            return div();
        }

        let children = std::mem::take(&mut props.children);
        let panel = div()
            .bg(panel_bg)
            .border_1()
            .border_color(panel_border)
            .p(pad)
            .rounded(r)
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .children(children)
            .shadow(vec![gpui::BoxShadow {
                color: gpui::hsla(0.0, 0.0, 0.0, alpha),
                blur_radius: gpui::px(12.0),
                spread_radius: gpui::px(0.0),
                offset: gpui::Point {
                    x: gpui::px(0.0),
                    y: gpui::px(4.0),
                },
            }]);

        div().child(AnimatedPresenceElement::new(
            props.state.clone(),
            props.id.clone(),
            SlideDirection::Down,
            px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32),
            panel,
        ))
    }
}

pub fn arc_modal<T: ModalRenderer + 'static>(r: T) -> Arc<dyn ModalRenderer> {
    Arc::new(r)
}
