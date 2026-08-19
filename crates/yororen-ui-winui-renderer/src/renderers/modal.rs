//! `WinUIModalRenderer` — default `ModalRenderer` impl.
//!
//! Paints the modal *panel* (bg, border, padding, radius, shadow).
//! The caller is responsible for the scrim / overlay and for adding
//! children inside the panel after `.render(cx)`.
//!
//! Visual spec follows the reference `WinContentDialog`:
//!
//! - fill `#FCFCFC` / `#2C2C2C`, 1px flyout stroke, 8px radius
//! - shadow `0 32px 64px rgba(0, 0, 0, 0.28)`
//! - 24px content padding
//! - enter: 250ms `cubic-bezier(0, 0, 0, 1)` fade + slide-up
//!   (the reference scales 1.05 → 1; gpui divs have no transform,
//!   so the slide-up approximates the same motion profile), exit
//!   167ms.

use std::sync::Arc;
use std::time::Duration;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div, px};

use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::modal::ModalProps;
use yororen_ui_core::theme::Theme;

use crate::animation::{AnimatedPresenceElement, fast_out_slow_in, motion_ms};

pub use yororen_ui_core::renderer::modal::{ModalRenderState, ModalRenderer};

pub struct WinUIModalRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIModalRenderer {
    pub fn panel_bg(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        // Reference ContentDialog fill: #FCFCFC light / #2C2C2C dark.
        theme
            .get_color("winui.dialog_bg")
            .or_else(|| theme.get_color("surface.raised"))
            .unwrap_or_default()
    }
    pub fn panel_border(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.flyout_stroke")
            .or_else(|| theme.get_color("border.muted"))
            .unwrap_or_default()
    }
    pub fn panel_padding(&self, _state: &ModalRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.modal.padding")
                .or_else(|| theme.get_number("tokens.spacing.inset_lg"))
                .unwrap_or(24.0) as f32,
        )
    }
    pub fn panel_border_radius(&self, _state: &ModalRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(8.0) as f32)
    }
    pub fn panel_shadow_alpha(&self, _state: &ModalRenderState, theme: &Theme) -> f32 {
        theme
            .get_color("shadow.dialog")
            .unwrap_or_else(|| gpui::hsla(0., 0., 0., 0.28))
            .a
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
                blur_radius: gpui::px(64.0),
                spread_radius: gpui::px(0.0),
                offset: gpui::Point {
                    x: gpui::px(0.0),
                    y: gpui::px(32.0),
                },
            }]);

        // ContentDialog motion: 250ms in / 167ms out with the
        // `cubic-bezier(0, 0, 0, 1)` curve, sliding *up* into place.
        let enter = yororen_ui_core::animation::AnimationConfig::new()
            .with_duration(Duration::from_millis(motion_ms(
                theme,
                "duration_modal_slide_up",
                250.0,
            )))
            .with_easing(fast_out_slow_in);
        let exit = yororen_ui_core::animation::AnimationConfig::new()
            .with_duration(Duration::from_millis(167))
            .with_easing(fast_out_slow_in);

        div().child(
            AnimatedPresenceElement::new(
                props.state.clone(),
                props.id.clone(),
                SlideDirection::Up,
                px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32),
                panel,
            )
            .with_configs(enter, exit),
        )
    }
}

pub fn arc_modal<T: ModalRenderer + 'static>(r: T) -> Arc<dyn ModalRenderer> {
    Arc::new(r)
}
