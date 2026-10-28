//! `TokenScrollViewRenderer` — default `ScrollViewRenderer` impl.
//!
//! Builds the scroll view out of three parts:
//!
//! 1. the **content area** — a full-size `overflow_y_scroll` /
//!    `overflow_x_scroll` div with `track_scroll(&handle)` that
//!    hosts the caller's children (gpui's own wheel handling does
//!    the actual scrolling);
//! 2. the **scrollbar overlays** — absolutely-positioned tracks on
//!    the right / bottom edge, each a flex column (or row) of
//!    `page-up zone | thumb | page-down zone`, so track clicks page
//!    without any window-coordinate math;
//! 3. the **wheel-propagation guard and keyboard scrolling** on
//!    the outer shell — the same guard recipe the virtual-list
//!    renderers use (`on_scroll_wheel`, `stop_propagation`, and
//!    `occlude()`), plus arrow / PgUp / PgDn / Home / End keys.
//!
//! Thumb geometry comes from the shared
//! [`ScrollViewRenderState::thumb_metrics`] and thumb dragging from
//! the shared [`ScrollThumbDrag`] payload (both in
//! `yororen-ui-core`), following the headless `slider` drag pattern
//! (`on_drag` + `on_drag_move` with a global mouse-follow drag).
//!
//! Metrics are read from the controller at compose time, i.e. from
//! the handle's last prepaint — one frame of lag at most, which is
//! the standard gpui scrollbar approach.

use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, CursorStyle, Div, ElementId, Empty, Hsla, InteractiveElement, KeyDownEvent,
    MouseButton, MouseDownEvent, ParentElement, Pixels, Stateful, StatefulInteractiveElement,
    Styled, div, hsla, point, px,
};

use yororen_ui_core::headless::scroll_view::{
    ScrollViewController, ScrollViewProps, ScrollViewScrollCallback, ScrollbarVisibility,
};
use yororen_ui_core::renderer::scroll_view::{
    ScrollThumbDrag, ScrollViewRenderState, ScrollViewRenderer,
};
use yororen_ui_core::theme::Theme;

pub struct TokenScrollViewRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenScrollViewRenderer {
    pub fn thumb_color(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("scrollbar.thumb")
            .unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.35))
    }
    pub fn thumb_hover_color(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("scrollbar.thumb_hover")
            .unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.5))
    }
    pub fn track_color(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Option<Hsla> {
        theme.get_color("scrollbar.track")
    }
    pub fn bar_width(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.scroll_view.bar_width")
            .unwrap_or(10.0) as f32)
    }
    pub fn thumb_min_size(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.scroll_view.thumb_min_size")
            .unwrap_or(24.0) as f32)
    }
    pub fn track_inset(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.scroll_view.track_inset")
            .unwrap_or(2.0) as f32)
    }
    pub fn nav_step(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.scroll_view.nav_step")
            .unwrap_or(48.0) as f32)
    }
    pub fn frame_bg(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn frame_border_color(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn frame_radius(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.sm").unwrap_or(4.0) as f32)
    }
}

/// Fire the `on_scroll` callback with the post-scroll offset, then
/// mark the window dirty so the overlay thumbs repaint.
fn notify_scrolled(
    controller: &ScrollViewController,
    on_scroll: &Option<ScrollViewScrollCallback>,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) {
    if let Some(cb) = on_scroll {
        cb(controller.offset(), controller.max_offset(), window, cx);
    }
    window.refresh();
}

impl ScrollViewRenderer for TokenScrollViewRenderer {
    fn compose(&self, props: ScrollViewProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let ScrollViewProps {
            id,
            controller,
            scroll_x,
            scroll_y,
            scrollbar_visibility,
            framed,
            max_content_height,
            gap,
            children,
            on_scroll,
            occlude,
        } = props;

        let theme = cx.theme();
        let rstate = ScrollViewRenderState {
            scroll_x,
            scroll_y,
            scrollbar_visibility,
            framed,
        };
        let bar_w = self.bar_width(&rstate, theme);
        let thumb_min = self.thumb_min_size(&rstate, theme);
        let inset = self.track_inset(&rstate, theme);
        let nav_step = self.nav_step(&rstate, theme);
        let thumb_color = self.thumb_color(&rstate, theme);
        let thumb_hover_color = self.thumb_hover_color(&rstate, theme);
        let track_color = self.track_color(&rstate, theme);

        // Metrics snapshot from the handle's last prepaint.
        let offset = controller.offset();
        let max = controller.max_offset();
        let vp = controller.viewport_size();
        let content = controller.content_size();

        let show_bars = scrollbar_visibility == ScrollbarVisibility::Always;
        let v_metrics = ScrollViewRenderState::thumb_metrics(
            vp.height - inset * 2.0,
            content.height,
            vp.height,
            offset.y,
            max.height,
            thumb_min,
        );
        let h_metrics = ScrollViewRenderState::thumb_metrics(
            vp.width - inset * 2.0,
            content.width,
            vp.width,
            offset.x,
            max.width,
            thumb_min,
        );
        let v_shown = show_bars && scroll_y && v_metrics.scrollable;
        let h_shown = show_bars && scroll_x && h_metrics.scrollable;

        let id_str = format!("{:?}", id);

        // ---- Content area ------------------------------------------------
        // Use flex layout only when we actually stack multiple children
        // (or need a gap). A single natural-size child — the TextArea
        // embed case — keeps block layout so it is NOT stretched to the
        // viewport height (stretching it would erase the overflow and
        // make the container non-scrollable).
        let single_block_child = gap.is_none() && children.len() == 1;
        let mut content_el = div()
            .id(ElementId::Name(format!("{id_str}-content").into()))
            .size_full();
        if !single_block_child {
            content_el = content_el.flex().flex_col();
        }
        if scroll_y {
            content_el = content_el.overflow_y_scroll();
        }
        if scroll_x {
            content_el = content_el.overflow_x_scroll();
        }
        content_el = content_el.track_scroll(&controller.handle());
        if let Some(max_h) = max_content_height {
            content_el = content_el.max_h(max_h);
        }
        if let Some(gap) = gap {
            content_el = content_el.gap(gap);
        }
        for child in children {
            content_el = content_el.child(child);
        }

        let mut outer = div().id(id).relative().overflow_hidden();
        if framed {
            outer = outer
                .bg(self.frame_bg(&rstate, theme))
                .border_1()
                .border_color(self.frame_border_color(&rstate, theme))
                .rounded(self.frame_radius(&rstate, theme));
        }
        outer = outer.child(content_el);

        // ---- Vertical scrollbar overlay ----------------------------------
        if v_shown {
            // Reserve the bottom-right corner when the horizontal
            // bar is also visible so the two tracks never cross.
            let bottom_reserve = if h_shown { inset * 2.0 + bar_w } else { inset };
            let track_len = vp.height - inset - bottom_reserve;

            let mut track = div()
                .id(ElementId::Name(format!("{id_str}-vtrack").into()))
                .absolute()
                .top(inset)
                .bottom(bottom_reserve)
                .right_0()
                .w(bar_w + inset)
                .flex()
                .flex_col()
                .items_end()
                .pr(inset)
                .block_mouse_except_scroll();
            if let Some(track_bg) = track_color {
                track = track.bg(track_bg).rounded_full();
            }

            // Page-up zone above the thumb.
            let controller_up = controller.clone();
            let on_scroll_up = on_scroll.clone();
            let page_step = vp.height;
            track = track.child(
                div()
                    .id(ElementId::Name(format!("{id_str}-vpageup").into()))
                    .h(v_metrics.thumb_start)
                    .w(bar_w)
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        move |_ev: &MouseDownEvent, window, cx| {
                            controller_up.scroll_by(point(px(0.), page_step));
                            notify_scrolled(&controller_up, &on_scroll_up, window, cx);
                        },
                    ),
            );

            // Draggable thumb.
            let drag = ScrollThumbDrag {
                controller: controller.clone(),
                vertical: true,
                track_len,
                thumb_len: v_metrics.thumb_len,
            };
            // Drag state lives on the caller-owned controller: the
            // renderer rebuilds this element every frame (and
            // `refresh()`es each move), so any frame-local anchor
            // would be wiped mid-drag. `begin_thumb_drag` records the
            // absolute cursor position + offset at drag start.
            let controller_start = controller.clone();
            let controller_end = controller.clone();
            let drag_value = drag.clone();
            let on_scroll_drag = on_scroll.clone();
            let thumb = div()
                .id(ElementId::Name(format!("{id_str}-vthumb").into()))
                .h(v_metrics.thumb_len)
                .w(bar_w)
                .rounded_full()
                .bg(thumb_color)
                .hover(move |s| s.bg(thumb_hover_color))
                .cursor(CursorStyle::ResizeUpDown)
                .on_drag(drag_value, move |_v, _start, window, cx| {
                    controller_start.begin_thumb_drag(window.mouse_position());
                    cx.new(|_| Empty)
                })
                .on_drag_move(
                    move |ev: &gpui::DragMoveEvent<ScrollThumbDrag>, window, cx| {
                        if let Some(o) = drag.apply_drag_move(ev.event.position)
                            && let Some(cb) = &on_scroll_drag
                        {
                            cb(o, drag.controller.max_offset(), window, cx);
                        }
                        window.refresh();
                    },
                )
                .on_mouse_up(MouseButton::Left, move |_ev, _window, _cx| {
                    controller_end.end_thumb_drag();
                });
            track = track.child(thumb);

            // Page-down zone below the thumb.
            let controller_down = controller.clone();
            let on_scroll_down = on_scroll.clone();
            let page_step = vp.height;
            track = track.child(
                div()
                    .id(ElementId::Name(format!("{id_str}-vpagedown").into()))
                    .flex_1()
                    .w(bar_w)
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        move |_ev: &MouseDownEvent, window, cx| {
                            controller_down.scroll_by(point(px(0.), -page_step));
                            notify_scrolled(&controller_down, &on_scroll_down, window, cx);
                        },
                    ),
            );
            outer = outer.child(track);
        }

        // ---- Horizontal scrollbar overlay --------------------------------
        if h_shown {
            let right_reserve = if v_shown { inset * 2.0 + bar_w } else { inset };
            let track_len = vp.width - inset - right_reserve;

            let mut track = div()
                .id(ElementId::Name(format!("{id_str}-htrack").into()))
                .absolute()
                .left(inset)
                .right(right_reserve)
                .bottom_0()
                .h(bar_w + inset)
                .flex()
                .flex_row()
                .items_end()
                .pb(inset)
                .block_mouse_except_scroll();
            if let Some(track_bg) = track_color {
                track = track.bg(track_bg).rounded_full();
            }

            let controller_up = controller.clone();
            let on_scroll_up = on_scroll.clone();
            let page_step = vp.width;
            track = track.child(
                div()
                    .id(ElementId::Name(format!("{id_str}-hpageleft").into()))
                    .w(h_metrics.thumb_start)
                    .h(bar_w)
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        move |_ev: &MouseDownEvent, window, cx| {
                            controller_up.scroll_by(point(page_step, px(0.)));
                            notify_scrolled(&controller_up, &on_scroll_up, window, cx);
                        },
                    ),
            );

            let drag = ScrollThumbDrag {
                controller: controller.clone(),
                vertical: false,
                track_len,
                thumb_len: h_metrics.thumb_len,
            };
            // Same as the vertical thumb: drag state lives on the
            // controller so it survives the per-instance rebuild.
            let controller_start = controller.clone();
            let controller_end = controller.clone();
            let drag_value = drag.clone();
            let on_scroll_drag = on_scroll.clone();
            let thumb = div()
                .id(ElementId::Name(format!("{id_str}-hthumb").into()))
                .w(h_metrics.thumb_len)
                .h(bar_w)
                .rounded_full()
                .bg(thumb_color)
                .hover(move |s| s.bg(thumb_hover_color))
                .cursor(CursorStyle::ResizeLeftRight)
                .on_drag(drag_value, move |_v, _start, window, cx| {
                    controller_start.begin_thumb_drag(window.mouse_position());
                    cx.new(|_| Empty)
                })
                .on_drag_move(
                    move |ev: &gpui::DragMoveEvent<ScrollThumbDrag>, window, cx| {
                        if let Some(o) = drag.apply_drag_move(ev.event.position)
                            && let Some(cb) = &on_scroll_drag
                        {
                            cb(o, drag.controller.max_offset(), window, cx);
                        }
                        window.refresh();
                    },
                )
                .on_mouse_up(MouseButton::Left, move |_ev, _window, _cx| {
                    controller_end.end_thumb_drag();
                });
            track = track.child(thumb);

            let controller_down = controller.clone();
            let on_scroll_down = on_scroll.clone();
            let page_step = vp.width;
            track = track.child(
                div()
                    .id(ElementId::Name(format!("{id_str}-hpageright").into()))
                    .flex_1()
                    .h(bar_w)
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        move |_ev: &MouseDownEvent, window, cx| {
                            controller_down.scroll_by(point(-page_step, px(0.)));
                            notify_scrolled(&controller_down, &on_scroll_down, window, cx);
                        },
                    ),
            );
            outer = outer.child(track);
        }

        // ---- Wheel guard + keyboard scrolling -----------------------------
        // `gpui`'s overflow_scroll consumes the wheel delta inside
        // the content area but does **not** call `stop_propagation()`
        // (same story as `gpui::list` — see the virtual-list
        // renderer). Without the explicit stop the event bubbles on
        // to any ancestor scrollable and scrolls that too. The
        // handler doubles as the `on_scroll` notification point:
        // by the time the bubble phase reaches the shell, the
        // content area's handler has already applied the delta.
        let controller_wheel = controller.clone();
        let on_scroll_wheel = on_scroll.clone();
        outer = outer.on_scroll_wheel(move |_ev, window, cx| {
            cx.stop_propagation();
            if let Some(cb) = &on_scroll_wheel {
                cb(
                    controller_wheel.offset(),
                    controller_wheel.max_offset(),
                    window,
                    cx,
                );
            }
        });

        let controller_keys = controller;
        let on_scroll_keys = on_scroll;
        outer
            .when(occlude, |s| s.occlude())
            .focusable()
            .on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let ks = &ev.keystroke;
                let viewport = controller_keys.viewport_size();
                let handled = match ks.key.as_str() {
                    "up" if scroll_y => {
                        controller_keys.scroll_by(point(px(0.), nav_step));
                        true
                    }
                    "down" if scroll_y => {
                        controller_keys.scroll_by(point(px(0.), -nav_step));
                        true
                    }
                    "left" if scroll_x => {
                        controller_keys.scroll_by(point(nav_step, px(0.)));
                        true
                    }
                    "right" if scroll_x => {
                        controller_keys.scroll_by(point(-nav_step, px(0.)));
                        true
                    }
                    "pageup" if scroll_y => {
                        controller_keys.scroll_by(point(px(0.), viewport.height));
                        true
                    }
                    "pagedown" if scroll_y => {
                        controller_keys.scroll_by(point(px(0.), -viewport.height));
                        true
                    }
                    "home" => {
                        if scroll_y {
                            controller_keys.scroll_to_top();
                        } else if scroll_x {
                            controller_keys.scroll_to_left();
                        }
                        scroll_y || scroll_x
                    }
                    "end" => {
                        if scroll_y {
                            controller_keys.scroll_to_bottom();
                        } else if scroll_x {
                            controller_keys.scroll_to_right();
                        }
                        scroll_y || scroll_x
                    }
                    _ => false,
                };
                if handled {
                    notify_scrolled(&controller_keys, &on_scroll_keys, window, cx);
                }
            })
    }
}

pub fn arc_scroll_view<T: ScrollViewRenderer + 'static>(r: T) -> Arc<dyn ScrollViewRenderer> {
    Arc::new(r)
}
