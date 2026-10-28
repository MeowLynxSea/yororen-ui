//! Brutalist `ScrollView` + `PipsPager` renderers.
//!
//! Same structure as the default renderer's implementations (see
//! `yororen-ui-default-renderer::renderers::{scroll_view,
//! pips_pager}` for the full commentary: content area +
//! `track_scroll`, zone-based scrollbar overlays, wheel guard,
//! keyboard scrolling, pip strip from `pip_slots`). The brutalist
//! treatment swaps the geometry: square thumbs (`BRUTAL_RADIUS`),
//! a chunkier bar, solid high-contrast fills, and the framed
//! variant takes the signature thick black border.

use std::sync::Arc;

use gpui::{
    App, AppContext, CursorStyle, Div, ElementId, Empty, Hsla, InteractiveElement, KeyDownEvent,
    MouseButton, MouseDownEvent, ParentElement, Pixels, Stateful, StatefulInteractiveElement,
    Styled, div, point, px,
};

use gpui::prelude::FluentBuilder;
use yororen_ui_core::headless::pips_pager::{PipSlot, PipsPagerOrientation, PipsPagerProps};
use yororen_ui_core::headless::scroll_view::{
    ScrollViewController, ScrollViewProps, ScrollViewScrollCallback, ScrollbarVisibility,
};
use yororen_ui_core::renderer::pips_pager::{PipsPagerRenderState, PipsPagerRenderer};
use yororen_ui_core::renderer::scroll_view::{
    ScrollThumbDrag, ScrollViewRenderState, ScrollViewRenderer,
};
use yororen_ui_core::theme::Theme;

use crate::style::{BRUTAL_RADIUS, brutal_border_color};

// =====================================================================
// ScrollView
// =====================================================================

pub struct BrutalScrollViewRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalScrollViewRenderer {
    pub fn thumb_color(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("scrollbar.thumb")
            .unwrap_or_else(|| brutal_border_color(theme))
    }
    pub fn thumb_hover_color(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or_else(|| brutal_border_color(theme))
    }
    pub fn track_color(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Option<Hsla> {
        theme.get_color("scrollbar.track")
    }
    pub fn bar_width(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.scroll_view.bar_width")
            .unwrap_or(14.0) as f32)
    }
    pub fn thumb_min_size(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.scroll_view.thumb_min_size")
            .unwrap_or(28.0) as f32)
    }
    pub fn track_inset(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.scroll_view.track_inset")
            .unwrap_or(3.0) as f32)
    }
    pub fn nav_step(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.scroll_view.nav_step")
            .unwrap_or(48.0) as f32)
    }
    pub fn frame_bg(&self, _state: &ScrollViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn frame_border_width(&self, _state: &ScrollViewRenderState, _: &Theme) -> Pixels {
        px(crate::style::BRUTAL_BORDER_WIDTH)
    }
    pub fn frame_radius(&self, _state: &ScrollViewRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
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

impl ScrollViewRenderer for BrutalScrollViewRenderer {
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
        // Flex only when stacking multiple children / gaps; a single
        // natural-size child (TextArea embed) keeps block layout so it
        // isn't stretched to viewport height (which would kill overflow).
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
                .border(self.frame_border_width(&rstate, theme))
                .border_color(brutal_border_color(theme))
                .rounded(self.frame_radius(&rstate, theme));
        }
        outer = outer.child(content_el);

        // ---- Vertical scrollbar overlay ----------------------------------
        if v_shown {
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
                track = track.bg(track_bg);
            }

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

            let drag = ScrollThumbDrag {
                controller: controller.clone(),
                vertical: true,
                track_len,
                thumb_len: v_metrics.thumb_len,
            };
            // Drag state lives on the caller-owned controller so it
            // survives the renderer's per-instance rebuild.
            let controller_start = controller.clone();
            let controller_end = controller.clone();
            let drag_value = drag.clone();
            let on_scroll_drag = on_scroll.clone();
            let thumb = div()
                .id(ElementId::Name(format!("{id_str}-vthumb").into()))
                .h(v_metrics.thumb_len)
                .w(bar_w)
                .rounded(px(BRUTAL_RADIUS))
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
                track = track.bg(track_bg);
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
            let controller_start = controller.clone();
            let controller_end = controller.clone();
            let drag_value = drag.clone();
            let on_scroll_drag = on_scroll.clone();
            let thumb = div()
                .id(ElementId::Name(format!("{id_str}-hthumb").into()))
                .w(h_metrics.thumb_len)
                .h(bar_w)
                .rounded(px(BRUTAL_RADIUS))
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
        // Same story as the virtual-list renderers: gpui's
        // overflow_scroll consumes the wheel delta but does not
        // `stop_propagation()`, so the shell stops it here after the
        // content area has applied it — and doubles as the
        // `on_scroll` notification point.
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

// =====================================================================
// PipsPager
// =====================================================================

pub struct BrutalPipsPagerRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalPipsPagerRenderer {
    pub fn active_color(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or_else(|| brutal_border_color(theme))
    }
    pub fn inactive_color(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("border.default")
            .unwrap_or_else(|| brutal_border_color(theme))
    }
    pub fn fg(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn dot_size(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.pips_pager.dot_size")
            .unwrap_or(8.0) as f32)
    }
    pub fn active_dot_size(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.pips_pager.active_dot_size")
            .unwrap_or(20.0) as f32)
    }
    pub fn gap(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.pips_pager.gap")
            .unwrap_or(8.0) as f32)
    }
    pub fn nav_button_size(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.pips_pager.nav_button_size")
            .unwrap_or(28.0) as f32)
    }
    pub fn dot_border_width(&self, _: &PipsPagerRenderState, _: &Theme) -> Pixels {
        px(crate::style::BRUTAL_SMALL_BORDER_WIDTH)
    }
}

impl PipsPagerRenderer for BrutalPipsPagerRenderer {
    fn compose(&self, props: &PipsPagerProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let read = props.state.read(cx);
        let state = PipsPagerRenderState {
            page_count: read.page_count,
            current_page: read.current_page,
            orientation: read.orientation,
            max_visible_pips: read.max_visible_pips,
            show_nav_buttons: read.show_nav_buttons,
            ellipsis_before_elides: read.ellipsis_before_elides(),
            ellipsis_after_elides: read.ellipsis_after_elides(),
        };
        let slots = read.pip_slots();
        let vertical = read.orientation == PipsPagerOrientation::Vertical;
        let state_for_click = props.state.clone();
        let focus_handle = read.focus_handle();
        let _ = read;

        let active = self.active_color(&state, theme);
        let inactive = self.inactive_color(&state, theme);
        let fg = self.fg(&state, theme);
        let hover_bg = self.hover_bg(&state, theme);
        let dot = self.dot_size(&state, theme);
        let active_dot = self.active_dot_size(&state, theme);
        let gap = self.gap(&state, theme);
        let nav_size = self.nav_button_size(&state, theme);
        let dot_bw = self.dot_border_width(&state, theme);

        let id_str = format!("{:?}", props.id);

        let mut strip: Div = if vertical {
            div().flex().flex_col()
        } else {
            div().flex().flex_row()
        }
        .gap(px(4.))
        .items_center();

        for slot in slots {
            match slot {
                PipSlot::Page(page) => {
                    let is_current = page == state.current_page;
                    let state_for_pip = state_for_click.clone();
                    let mut pip = div()
                        .id(ElementId::Name(format!("{id_str}-pip-{page}").into()))
                        // Brutalist pips are square: solid fill when
                        // active, thick outline when idle.
                        .rounded(px(BRUTAL_RADIUS))
                        .cursor(CursorStyle::PointingHand)
                        .when(is_current, |p| p.bg(active))
                        .when(!is_current, |p| {
                            p.bg(gpui::hsla(0.0, 0.0, 0.0, 0.0))
                                .border(dot_bw)
                                .border_color(inactive)
                        });
                    if vertical {
                        pip = pip.w(dot).h(if is_current { active_dot } else { dot });
                    } else {
                        pip = pip.h(dot).w(if is_current { active_dot } else { dot });
                    }
                    pip = pip.on_click(move |_ev, window, cx| {
                        state_for_pip.update(cx, |s, cx_inner| {
                            s.go_to(page, window, &mut *cx_inner);
                        });
                    });
                    strip = strip.child(pip);
                }
                PipSlot::EllipsisBefore | PipSlot::EllipsisAfter => {
                    let before = slot == PipSlot::EllipsisBefore;
                    // Edge slots are rendered but inert (empty run) —
                    // kept only so the control width stays constant.
                    let elides = if before {
                        state.ellipsis_before_elides
                    } else {
                        state.ellipsis_after_elides
                    };
                    let state_for_ellipsis = state_for_click.clone();
                    let mut ellipsis = div()
                        .id(ElementId::Name(
                            format!(
                                "{id_str}-ellipsis-{}",
                                if before { "before" } else { "after" }
                            )
                            .into(),
                        ))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(inactive)
                        .text_size(dot * 1.2)
                        .when(vertical, |e| e.h(active_dot).w(dot))
                        .when(!vertical, |e| e.w(active_dot).h(dot))
                        .child("⋯");
                    if elides {
                        ellipsis = ellipsis.cursor(CursorStyle::PointingHand).on_click(
                            move |_ev, window, cx| {
                                state_for_ellipsis.update(cx, |s, cx_inner| {
                                    if before {
                                        s.skip_backward(window, &mut *cx_inner);
                                    } else {
                                        s.skip_forward(window, &mut *cx_inner);
                                    }
                                });
                            },
                        );
                    } else {
                        ellipsis = ellipsis.opacity(0.3);
                    }
                    strip = strip.child(ellipsis);
                }
            }
        }

        // Nav buttons — square bordered buttons with the signature
        // thick outline. Disabled-looking (faded, no-op) at the
        // ends; the headless `prev` / `next` already refuse to wrap.
        let make_nav = |label: &'static str, forward: bool, enabled: bool| {
            let state_for_nav = state_for_click.clone();
            div()
                .id(ElementId::Name(
                    format!("{id_str}-nav-{}", if forward { "next" } else { "prev" }).into(),
                ))
                .size(nav_size)
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(BRUTAL_RADIUS))
                .border_2()
                .border_color(inactive)
                .text_color(fg)
                .text_size(dot * 1.6)
                .when(enabled, |b| {
                    b.cursor(CursorStyle::PointingHand)
                        .hover(move |s| s.bg(hover_bg))
                })
                .when(!enabled, |b| b.opacity(0.35))
                .on_click(move |_ev, window, cx| {
                    state_for_nav.update(cx, |s, cx_inner| {
                        if forward {
                            s.next(window, &mut *cx_inner);
                        } else {
                            s.prev(window, &mut *cx_inner);
                        }
                    });
                })
                .child(label)
        };

        let at_first = state.current_page == 0 || state.page_count == 0;
        let at_last = state.page_count == 0 || state.current_page + 1 >= state.page_count;
        let prev_label = if vertical { "▴" } else { "‹" };
        let next_label = if vertical { "▾" } else { "›" };
        let prev = make_nav(prev_label, false, !at_first);
        let next = make_nav(next_label, true, !at_last);

        // Keyboard nav — same methods as the click handlers.
        // `window.refresh()` is required because `compose` runs
        // outside a paint context; without it a state-only update
        // never redraws (see the grid-view renderer for the full
        // rationale).
        let state_for_keys = state_for_click;
        div()
            .id(props.id.clone())
            .track_focus(&focus_handle)
            .flex()
            .items_center()
            .gap(gap)
            .when(vertical, |s| s.flex_col())
            .when(state.show_nav_buttons, |s| s.child(prev))
            .child(strip)
            .when(state.show_nav_buttons, |s| s.child(next))
            .on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let ks = &ev.keystroke;
                let handled = match ks.key.as_str() {
                    "right" if !vertical => {
                        state_for_keys.update(cx, |s, cx_inner| s.next(window, &mut *cx_inner));
                        true
                    }
                    "left" if !vertical => {
                        state_for_keys.update(cx, |s, cx_inner| s.prev(window, &mut *cx_inner));
                        true
                    }
                    "down" if vertical => {
                        state_for_keys.update(cx, |s, cx_inner| s.next(window, &mut *cx_inner));
                        true
                    }
                    "up" if vertical => {
                        state_for_keys.update(cx, |s, cx_inner| s.prev(window, &mut *cx_inner));
                        true
                    }
                    "home" => {
                        state_for_keys.update(cx, |s, cx_inner| {
                            s.go_to(0, window, &mut *cx_inner);
                        });
                        true
                    }
                    "end" => {
                        state_for_keys.update(cx, |s, cx_inner| {
                            let last = s.page_count.saturating_sub(1);
                            s.go_to(last, window, &mut *cx_inner);
                        });
                        true
                    }
                    _ => false,
                };
                if handled {
                    window.refresh();
                }
            })
    }
}

pub fn arc_pips_pager<T: PipsPagerRenderer + 'static>(r: T) -> Arc<dyn PipsPagerRenderer> {
    Arc::new(r)
}
