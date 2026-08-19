//! `WinUIVirtualListRenderer` — default `VirtualListRenderer` impl.
//!
//! Wraps `gpui::list(state, render_row)` in an outer `Div` that
//! gives it a stable id, theme-derived background / border /
//! radius, and `overflow_hidden` so the rounded corners clip
//! the scrolled content.
//!
//! The inner list is forced to `size_full().flex_grow().min_h_0()`
//! so it fills the parent — without this, `gpui::list` collapses
//! to zero height when nested in a flex column.
//!
//! If the headless props carry an `on_visible_range_change`
//! callback, we install it on the `ListState` via
//! `set_scroll_handler` before constructing the list element. The
//! handler is last-wins inside `ListState`, so re-installing on
//! every frame is correct (and cheap — one `Box::new` per frame).

use std::sync::Arc;

use gpui::{
    App, Div, Hsla, InteractiveElement, ParentElement, Pixels, Stateful, Styled, div, list, px,
};

use yororen_ui_core::headless::virtual_list::{RenderRowFn, VirtualListProps};
use yororen_ui_core::renderer::virtual_list::{VirtualListRenderState, VirtualListRenderer};
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub struct WinUIVirtualListRenderer;

impl WinUIVirtualListRenderer {
    pub fn bg(&self, _state: &VirtualListRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border_color(&self, _state: &VirtualListRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn border_radius(&self, _state: &VirtualListRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.sm").unwrap_or(4.0) as f32)
    }
}

impl VirtualListRenderer for WinUIVirtualListRenderer {
    fn compose(
        &self,
        mut props: VirtualListProps,
        render_row: RenderRowFn,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = VirtualListRenderState {
            item_count: props.item_count,
            alignment: props.alignment,
            overdraw: props.overdraw,
            sizing_behavior: props.sizing_behavior,
        };
        let bg = self.bg(&state, theme);
        let border = self.border_color(&state, theme);
        let radius = self.border_radius(&state, theme);

        // If the headless layer provided a visible-range callback,
        // wire it through `ListState::set_scroll_handler`. The
        // handler is `Some(Box<…>)` inside `ListState` — last-wins
        // on re-install, so calling this on every render frame is
        // correct: the new Box owns the caller's `FnMut`, and the
        // previous frame's handler is dropped at this point.
        if let Some(mut cb) = props.on_visible_range_change.take() {
            props.state.set_scroll_handler(move |ev, window, cx_inner| {
                cb(ev.visible_range.clone(), ev.count, window, cx_inner);
            });
        }

        // Construct the inner list with the caller's state and
        // sizing behavior, then force it to fill the parent —
        // `gpui::list` alone collapses to zero height in a
        // flex column without an explicit size.
        let list_el = list(props.state, render_row).with_sizing_behavior(props.sizing_behavior);
        let inner = list_el.size_full().flex_grow().min_h_0();

        // `gpui::list` handles scroll internally (its own
        // bubble-phase `on_mouse_event` consumes the delta and
        // calls `list_state.scroll(...)`), but it does **not**
        // call `stop_propagation()`. Without an explicit stop
        // here, the wheel event continues to bubble up to the
        // page / outer scrollable container and scrolls *that*
        // in addition to the list — the wrapping div
        // introduced this regression (the inner styled list was
        // itself the element previously, so there was no
        // outer hitbox to bubble past). Register a bubble-phase
        // scroll handler on the outer div that stops the event
        // after the list has handled it, and `occlude()` the
        // wrapper so hitboxes behind the list (e.g. an outer
        // page scroller) are not considered for scroll handling.
        div()
            .id(props.id)
            .flex()
            .flex_col()
            .bg(bg)
            .border_1()
            .border_color(border)
            .rounded(radius)
            .overflow_hidden()
            .child(inner)
            .on_scroll_wheel(|_event, _window, cx| {
                cx.stop_propagation();
            })
            .occlude()
    }
}

pub fn arc_virtual_list<T: VirtualListRenderer + 'static>(r: T) -> Arc<dyn VirtualListRenderer> {
    Arc::new(r)
}
