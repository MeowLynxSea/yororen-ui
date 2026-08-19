//! `WinUIOverlayRenderer` — default `OverlayRenderer` impl.
//!
//! Paints a *relative* scrim. The headless `OverlayProps::render(cx)`
//! layers dismissal handlers and visibility on top.
//!
//! `compose` returns a `relative().size_full()` div so the
//! overlay stays inside its parent's box (no `absolute().inset_0()`
//! that would escape to the document root). Visibility is
//! driven by `props.open` — when `false`, the scrim is
//! `invisible()` so the cell still shows the content underneath.

use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{App, Div, Hsla, InteractiveElement, ParentElement, Stateful, Styled, div};

use yororen_ui_core::headless::overlay::OverlayProps;
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::animation::fade_in_on_mount;

pub use yororen_ui_core::renderer::overlay::{OverlayRenderState, OverlayRenderer};

pub struct WinUIOverlayRenderer;

impl WinUIOverlayRenderer {
    pub fn scrim_color(&self, _state: &OverlayRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.scrim")
            // Reference dialog overlay: rgba(0, 0, 0, 0.30).
            .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.0, 0.30))
    }
}

impl OverlayRenderer for WinUIOverlayRenderer {
    fn compose(&self, props: &OverlayProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = OverlayRenderState { open: props.open };
        let scrim = self.scrim_color(&state, theme);

        let scrim_el = div()
            .relative()
            .size_full()
            .bg(scrim)
            .when(!props.open, |el| el.invisible());

        if !props.open {
            return div().id(props.id.clone()).child(scrim_el);
        }

        let duration_ms = theme
            .get_number("motion.duration_modal_fade")
            .unwrap_or(200.0) as u64;
        let el = fade_in_on_mount(
            scrim_el,
            props.id.clone(),
            std::time::Duration::from_millis(duration_ms),
            crate::animation::fast_out_slow_in,
        );
        div().id(props.id.clone()).child(el)
    }
}

pub fn arc_overlay<T: OverlayRenderer + 'static>(r: T) -> Arc<dyn OverlayRenderer> {
    Arc::new(r)
}
