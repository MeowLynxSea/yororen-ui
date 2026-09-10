//! `WinUIDropdownMenuRenderer` — default `DropdownMenuRenderer` impl.
//!
//! Composes the dropdown shell: trigger in normal flow, then
//! (when `state.is_open()`) the body floated with
//! `gpui::deferred` + absolute positioning so it paints on
//! top of subsequent sibling cells in the gallery.

use std::sync::Arc;
use std::time::Duration;

use gpui::{App, Div, Hsla, InteractiveElement, ParentElement, Pixels, Styled, div, px};

use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::dropdown_menu::DropdownMenuProps;
use yororen_ui_core::theme::Theme;

use crate::animation::{AnimatedPresenceElement, flyout_in, flyout_out, motion_ms};

pub use yororen_ui_core::renderer::dropdown_menu::{DropdownMenuRenderState, DropdownMenuRenderer};

pub struct WinUIDropdownMenuRenderer;

impl WinUIDropdownMenuRenderer {
    pub fn border(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.flyout_stroke")
            .or_else(|| theme.get_color("border.default"))
            .unwrap_or_default()
    }
    /// Flyout corner: 8px OverlayCornerRadius.
    pub fn border_radius(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(8.0) as f32)
    }
    pub fn padding(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32)
    }
    pub fn shadow_alpha(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> f32 {
        theme
            .get_color("shadow.flyout")
            .or_else(|| theme.get_color("shadow.elevation_2"))
            .unwrap_or_default()
            .alpha
    }
}

impl DropdownMenuRenderer for WinUIDropdownMenuRenderer {
    fn compose(&self, props: &mut DropdownMenuProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DropdownMenuRenderState {
            open: props.state.read(cx).is_open(),
        };
        let r = self.border_radius(&state, theme);
        let alpha = self.shadow_alpha(&state, theme);

        // Outer container is `relative` so the absolute panel
        // below is positioned relative to it.
        let mut outer = div().relative();

        // 1) Trigger — always rendered in normal flow.
        if let Some(t) = props.trigger.take() {
            outer = outer.child(t);
        }

        // 2) Body — only when visible, floated with
        //    `gpui::deferred` so it paints over subsequent
        //    sibling cells in the gallery.
        if props.state.read(cx).is_visible()
            && let Some(c) = props.content.take()
        {
            // The body is a `menu` element which already paints
            // its own border + bg; the dropdown panel only adds
            // shadow and click-outside dismissal. Avoid double
            // borders by NOT setting `border_1` / `border_color`
            // here.
            let state_for_close = props.state.clone();
            let panel: Div = div()
                .absolute()
                .top(gpui::px(4.0))
                .left_0()
                .text_color(theme.get_color("content.primary").unwrap_or_default())
                .rounded(r)
                .shadow(vec![gpui::BoxShadow {
                    color: gpui::hsla(0.0, 0.0, 0.0, alpha),
                    blur_radius: gpui::px(15.0),
                    spread_radius: gpui::px(0.0),
                    inset: false,
                    offset: gpui::Point {
                        x: gpui::px(0.0),
                        y: gpui::px(5.0),
                    },
                }])
                .occlude()
                .on_mouse_down_out(move |_ev, _window, cx| {
                    state_for_close.update(cx, |s, _cx| s.close());
                })
                .child(c);
            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            // WinUI flyout open/close: 250ms in / 100ms out.
            let enter = yororen_ui_core::animation::AnimationConfig::new()
                .with_duration(Duration::from_millis(motion_ms(
                    theme,
                    "duration_menu_open_slow",
                    250.0,
                )))
                .with_easing(flyout_in);
            let exit = yororen_ui_core::animation::AnimationConfig::new()
                .with_duration(Duration::from_millis(motion_ms(
                    theme,
                    "duration_menu_open_fast",
                    100.0,
                )))
                .with_easing(flyout_out);
            // The animation wrapper is absolutely positioned at the
            // top-left of the outer relative container so the panel
            // inside keeps its original `top/left` offset.
            outer = outer.child(
                gpui::deferred(
                    div().absolute().top_0().left_0().child(
                        AnimatedPresenceElement::new(
                            props.state.clone(),
                            (props.id.clone(), "body"),
                            SlideDirection::Down,
                            distance,
                            panel,
                        )
                        .with_configs(enter, exit),
                    ),
                )
                .with_priority(1),
            );
        }

        outer
    }
}

pub fn arc_dropdown_menu<T: DropdownMenuRenderer + 'static>(r: T) -> Arc<dyn DropdownMenuRenderer> {
    Arc::new(r)
}
