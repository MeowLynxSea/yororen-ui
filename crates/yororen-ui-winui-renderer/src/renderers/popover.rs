//! `WinUIPopoverRenderer` — default `PopoverRenderer` impl.
//!
//! Composes the popover shell: trigger in normal flow, then
//! (when `state.is_open()`) the content floated with
//! `gpui::deferred` + absolute positioning so it paints on
//! top of subsequent sibling cells in the gallery.

use std::sync::Arc;
use std::time::Duration;

use gpui::{App, Div, Hsla, InteractiveElement, ParentElement, Pixels, Styled, div, px};

use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::popover::PopoverProps;
use yororen_ui_core::theme::Theme;

use crate::animation::{AnimatedPresenceElement, flyout_in, flyout_out, motion_ms};

pub use yororen_ui_core::renderer::popover::{PopoverRenderState, PopoverRenderer};

pub struct WinUIPopoverRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIPopoverRenderer {
    pub fn bg(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.flyout_bg")
            .or_else(|| theme.get_color("surface.popover"))
            .unwrap_or_default()
    }
    pub fn border(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.flyout_stroke")
            .or_else(|| theme.get_color("border.muted"))
            .unwrap_or_default()
    }
    pub fn shadow_alpha(&self, _state: &PopoverRenderState, theme: &Theme) -> f32 {
        theme
            .get_color("shadow.flyout")
            .or_else(|| theme.get_color("shadow.elevation_2"))
            .unwrap_or_default()
            .a
    }
    /// Flyouts use the 8px OverlayCornerRadius.
    pub fn border_radius(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(8.0) as f32)
    }
    pub fn offset(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.popover.offset")
                .unwrap_or(0.0) as f32,
        )
    }
}

impl PopoverRenderer for WinUIPopoverRenderer {
    fn compose(&self, props: &mut PopoverProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = PopoverRenderState {};
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let r = self.border_radius(&state, theme);
        let alpha = self.shadow_alpha(&state, theme);
        let visible = props.state.read(cx).is_visible();
        let offset_px = self.offset(&state, theme);

        // Outer container is `relative` so the absolute panel
        // below is positioned relative to it.
        let mut outer = div().relative();

        // 1) Trigger — always rendered in normal flow.
        //    Clicking the trigger also closes the popover when
        //    it is already open (the trigger button toggles
        //    state; the next render with `open=true` lets
        //    outside-click close it).
        if let Some(t) = props.trigger.take() {
            outer = outer.child(t);
        }

        // 2) Content — only when visible, floated with
        //    `gpui::deferred` so it paints after the
        //    subsequent sibling cells in the gallery.
        if visible && let Some(c) = props.content.take() {
            // The outer container captures outside-clicks and
            // closes the popover; the floating panel calls
            // `.occlude()` to swallow hits for elements behind
            // it, so an outside-click only fires when the user
            // actually clicks outside the panel.
            let state_for_close = props.state.clone();
            outer = outer.on_mouse_down_out(move |_ev, _window, cx| {
                state_for_close.update(cx, |s, _cx| s.close());
            });
            let panel: Div = div()
                .absolute()
                .top(offset_px)
                .left_0()
                // Absolute panels don't inherit an intrinsic width
                // from the trigger — floor it so text content never
                // collapses to a character-wide column (reference
                // flyouts are `width: max-content; min-width: 120px`).
                .min_w(px(theme
                    .get_number("tokens.control.popover.min_width")
                    .unwrap_or(180.0) as f32))
                .max_w(px(theme
                    .get_number("tokens.control.popover.max_width")
                    .unwrap_or(360.0) as f32))
                .bg(bg)
                .text_color(theme.get_color("content.primary").unwrap_or_default())
                .border_1()
                .border_color(border)
                .rounded(r)
                .shadow(vec![gpui::BoxShadow {
                    color: gpui::hsla(0.0, 0.0, 0.0, alpha),
                    blur_radius: gpui::px(15.0),
                    spread_radius: gpui::px(0.0),
                    offset: gpui::Point {
                        x: gpui::px(0.0),
                        y: gpui::px(5.0),
                    },
                }])
                .occlude()
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
                            (props.id.clone(), "content"),
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

pub fn arc_popover<T: PopoverRenderer + 'static>(r: T) -> Arc<dyn PopoverRenderer> {
    Arc::new(r)
}
