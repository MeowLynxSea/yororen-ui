//! `ScrollViewRenderer` — visual contract for `ScrollView`.
//!
//! Like [`crate::renderer::virtual_list`]'s trait, `compose`
//! **consumes** the `ScrollViewProps` because the content children
//! are owned by the props and must be transferred into the
//! renderer's scrollable content area. The renderer is responsible
//! for:
//!
//! 1. Reading theme tokens (scrollbar thumb/track colours, frame
//!    surface, radii) from `cx.theme()`.
//! 2. Building the content area (`overflow_y_scroll` /
//!    `overflow_x_scroll` + `track_scroll(&controller.handle())`)
//!    and the scrollbar overlay (thumb sized from
//!    `viewport / content`, positioned from `-offset / max_offset`).
//! 3. Wiring thumb drag, track-click paging, keyboard scrolling,
//!    and the wheel-propagation guard (`on_scroll_wheel` +
//!    `stop_propagation` + `occlude`, matching the virtual-list
//!    renderers), plus firing the props' `on_scroll` callback.
//! 4. Returning a `Stateful<Div>` the caller can further style
//!    (typically `.w(...).h(...)`).

use std::any::Any;

use gpui::{App, Div, Pixels, Point, Stateful, point};

use crate::headless::scroll_view::{ScrollViewController, ScrollViewProps, ScrollbarVisibility};

/// Pure data view of a scroll view, derived from
/// [`ScrollViewProps`] at render time. Renderer helpers consume
/// this to look up theme tokens without having to re-derive
/// everything.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollViewRenderState {
    pub scroll_x: bool,
    pub scroll_y: bool,
    pub scrollbar_visibility: ScrollbarVisibility,
    pub framed: bool,
}

/// Thumb geometry for one scrollbar axis, derived from the
/// controller each frame. The formula lives here — not in each
/// renderer — so all renderers draw thumbs that agree on size and
/// position; only colours / radii / insets differ per theme.
///
/// `scrollable` is false (and the geometry zero) when the axis has
/// no overflow, in which case the renderer skips drawing the axis.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollbarMetrics {
    pub scrollable: bool,
    /// Thumb length in pixels along the track.
    pub thumb_len: Pixels,
    /// Thumb start offset in pixels from the track's start.
    pub thumb_start: Pixels,
}

impl ScrollViewRenderState {
    /// Derive one axis's thumb metrics. Pure arithmetic over the
    /// handle's last-prepaint state:
    ///
    /// - `track_len` — drawable track length (viewport extent minus
    ///   any renderer-side inset);
    /// - `content` / `viewport` — the axis's content and viewport
    ///   extents from the controller;
    /// - `offset` / `max` — the axis's scroll offset (≤ 0) and max
    ///   offset (≥ 0);
    /// - `min_thumb` — smallest thumb the theme allows.
    pub fn thumb_metrics(
        track_len: Pixels,
        content: Pixels,
        viewport: Pixels,
        offset: Pixels,
        max: Pixels,
        min_thumb: Pixels,
    ) -> ScrollbarMetrics {
        if content <= viewport || max <= Pixels::ZERO || track_len <= Pixels::ZERO {
            return ScrollbarMetrics::default();
        }
        let ratio = (viewport / content).clamp(0., 1.);
        let thumb_len = (track_len * ratio).max(min_thumb);
        let scrollable_track = track_len - thumb_len;
        // offset ≤ 0, so -offset / max ∈ [0, 1] is how far along
        // the content the viewport sits.
        let progress = (-offset / max).clamp(0., 1.);
        let thumb_start = scrollable_track * progress;
        ScrollbarMetrics {
            scrollable: true,
            thumb_len,
            thumb_start,
        }
    }
}

pub trait ScrollViewRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` scrollable container. The
    /// props are consumed so the children, the controller, and the
    /// `on_scroll` callback can be moved into the renderer's
    /// content area and event handlers.
    fn compose(&self, props: ScrollViewProps, cx: &App) -> Stateful<Div>;
}

/// Payload for a scrollbar-thumb drag, shared by every renderer so
/// the cursor-delta → scroll-offset mapping cannot drift between
/// themes. One payload instance is captured at render time (the
/// frame the drag starts in is at most one frame stale); the
/// renderers hand it to `on_drag` / `on_drag_move` following the
/// headless `slider` drag pattern.
#[derive(Clone, Debug)]
pub struct ScrollThumbDrag {
    pub controller: ScrollViewController,
    /// True for the vertical thumb, false for the horizontal one.
    pub vertical: bool,
    /// Track inner length in pixels (viewport extent minus
    /// renderer insets).
    pub track_len: Pixels,
    /// Thumb length in pixels.
    pub thumb_len: Pixels,
}

impl ScrollThumbDrag {
    /// Map a cursor delta (along the axis, in pixels) to the target
    /// axis offset, starting from the drag-start offset. Moving the
    /// thumb down / right scrolls further into the content (offset
    /// grows more negative). Unscrollable axes (or degenerate track
    /// geometry) pin to the start offset.
    pub fn offset_for_delta(&self, start_offset: Pixels, delta: Pixels) -> Pixels {
        let max = if self.vertical {
            self.controller.max_offset().height
        } else {
            self.controller.max_offset().width
        };
        let scrollable_track = self.track_len - self.thumb_len;
        if scrollable_track <= Pixels::ZERO || max <= Pixels::ZERO {
            return start_offset;
        }
        let px_per_track_px = max / scrollable_track;
        start_offset - delta * px_per_track_px
    }

    /// Apply one drag move: read the drag-start state from the
    /// controller, measure the cursor delta against the stored
    /// anchor, and scroll. Returns the post-scroll offset so the
    /// renderer can forward it to `on_scroll`. `None` when no drag
    /// is in progress (before gpui's drag-start threshold).
    pub fn apply_drag_move(&self, cursor: Point<Pixels>) -> Option<Point<Pixels>> {
        let state = self.controller.thumb_drag_state()?;
        let delta = if self.vertical {
            cursor.y - state.anchor.y
        } else {
            cursor.x - state.anchor.x
        };
        let start_axis = if self.vertical {
            state.start_offset.y
        } else {
            state.start_offset.x
        };
        let axis = self.offset_for_delta(start_axis, delta);
        let o = self.controller.offset();
        let target = if self.vertical {
            point(o.x, axis)
        } else {
            point(axis, o.y)
        };
        self.controller.scroll_to(target);
        Some(self.controller.offset())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn unscrollable_axis_has_no_geometry() {
        let m = ScrollViewRenderState::thumb_metrics(
            px(100.),
            px(100.),
            px(100.),
            px(0.),
            px(0.),
            px(20.),
        );
        assert!(!m.scrollable);
        assert_eq!(m.thumb_len, px(0.));
    }

    #[test]
    fn thumb_covers_half_of_a_double_length_track() {
        // Content exactly 2× the viewport → thumb is half the
        // track.
        let m = ScrollViewRenderState::thumb_metrics(
            px(100.),
            px(200.),
            px(100.),
            px(0.),
            px(100.),
            px(20.),
        );
        assert!(m.scrollable);
        assert_eq!(m.thumb_len, px(50.));
        assert_eq!(m.thumb_start, px(0.), "at the top edge");
    }

    #[test]
    fn thumb_respects_minimum_and_tracks_progress() {
        // Content 10× the viewport → naive thumb would be 10px,
        // clamped up to the 24px minimum.
        let m = ScrollViewRenderState::thumb_metrics(
            px(100.),
            px(1000.),
            px(100.),
            px(0.),
            px(900.),
            px(24.),
        );
        assert_eq!(m.thumb_len, px(24.));
        // Half-scrolled (offset -450 of max 900) → thumb sits at
        // the midpoint of the remaining track (100 - 24) / 2 = 38.
        let m = ScrollViewRenderState::thumb_metrics(
            px(100.),
            px(1000.),
            px(100.),
            px(-450.),
            px(900.),
            px(24.),
        );
        assert_eq!(m.thumb_start, px(38.));
    }
}
