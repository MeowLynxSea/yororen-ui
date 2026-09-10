//! Material Design-styled button with a click ripple animation.
//!
//! A custom [`gpui::Element`] paints an expanding, semi-transparent
//! circle that starts at the click point and grows to cover the
//! button — the classic M2 ripple. The fill is white at a small
//! peak alpha; as the circle grows, the alpha fades linearly to 0
//! so the ripple dissolves into the background. The button is
//! wired through the headless `button` factory for a11y (focus +
//! click), but every visual decision (background, radius,
//! typography, ripple color) lives with the caller.
//!
//! ## How the animation is driven
//!
//! 1. The click handler on the button div captures the
//!    window-space `MouseDownEvent::position` and pushes a new
//!    `Ripple` (window-space center + start time) into the
//!    `RippleState` entity.
//! 2. The `RippleElement::prepaint` reads the state, computes
//!    each ripple's current radius and alpha, and asks the
//!    window to redraw on the next animation frame via
//!    `window.request_animation_frame()` — only while there are
//!    live ripples.
//! 3. `RippleElement::paint` paints a `PaintQuad` per ripple. The
//!    next frame's `prepaint` cleans up finished ripples and
//!    computes new paint data.
//!
//! The button's own `overflow: hidden` clips each ripple to the
//! rounded button shape, so no extra clipping element is needed.

use std::time::{Duration, Instant};

use gpui::{
    AbsoluteLength, App, Bounds, DefiniteLength, Div, Edges, Element, ElementId, Entity,
    GlobalElementId, Hsla, InspectorElementId, InteractiveElement, IntoElement, LayoutId, Length,
    MouseButton, MouseDownEvent, ParentElement, Path, PathBuilder, Pixels, Point, Position,
    SharedString, Stateful, Style, Styled, Window, div, hsla, point, px,
};
use yororen_ui::headless::button::button;

/// Total ripple animation duration. Material's spec uses 300–500ms
/// depending on the surface size; 450ms feels right for a 36px
/// raised button.
const MD_RIPPLE_DURATION: Duration = Duration::from_millis(450);

/// Padding added to the ripple's final radius so it fully covers
/// the button's corners (the diagonal isn't always enough — if
/// the user clicks near an edge, the farthest corner is further
/// than the geometric diagonal).
const MD_RIPPLE_CORNER_PADDING: f32 = 10.0;

/// Peak alpha for the ripple fill. MD2 uses ~0.16 black on light
/// surfaces; we use white at 0.20 to be visible on the teal
/// background without overpowering the label.
const MD_RIPPLE_PEAK_ALPHA: f32 = 0.20;

/// A zero-length `Length` for `Edges::all`. Used to pin an
/// absolutely-positioned element to all four sides of its
/// containing block (the "fill the parent" pattern).
const ZERO_LENGTH: Length = Length::Definite(DefiniteLength::Absolute(AbsoluteLength::Pixels(
    gpui::px(0.),
)));

/// One click's worth of ripple. `center` is the click point in
/// **window coordinates** — the same space as the bounds
/// `prepaint` receives, so no translation is needed at paint
/// time.
#[derive(Clone, Copy)]
struct Ripple {
    center: Point<Pixels>,
    started: Instant,
}

/// Persistent state for the ripple element. Minted via
/// `window.use_keyed_state` so it survives across paints. The
/// `use_keyed_state` machinery also installs an observer that
/// re-renders the current view whenever this entity is updated —
/// that's what triggers a re-render after the click handler
/// pushes a new ripple.
#[derive(Default)]
struct RippleState {
    ripples: Vec<Ripple>,
}

impl RippleState {
    fn push(&mut self, click: Point<Pixels>) {
        self.ripples.push(Ripple {
            center: click,
            started: Instant::now(),
        });
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        self.ripples
            .retain(|r| now.duration_since(r.started) < MD_RIPPLE_DURATION);
    }
}

/// Pre-computed paint data for one ripple. Computed in `prepaint`,
/// consumed in `paint`. The `Path` is built from four quarter
/// arcs; we can't use `paint_quad` here because that fills a
/// *rectangle*, not a circle — a square clip at full radius
/// shows up as a block of color, not a round ripple.
struct RipplePaint {
    path: Path<Pixels>,
    color: Hsla,
}

/// Build a circular path centered at `center` with radius
/// `radius`, in window coordinates. The path is four quarter
/// arcs (east → south → west → north → east).
fn circle_path(center: Point<Pixels>, radius: Pixels) -> Path<Pixels> {
    let mut builder = PathBuilder::fill();
    let r = radius;
    let cx = center.x;
    let cy = center.y;
    builder.move_to(point(cx + r, cy));
    builder.arc_to(point(r, r), px(0.), false, true, point(cx, cy + r));
    builder.arc_to(point(r, r), px(0.), false, true, point(cx - r, cy));
    builder.arc_to(point(r, r), px(0.), false, true, point(cx, cy - r));
    builder.arc_to(point(r, r), px(0.), false, true, point(cx + r, cy));
    builder.close();
    builder.build().expect("valid circle path")
}

/// Custom `gpui::Element` that paints the ripple circles. Sized
/// `absolute inset 0` so it overlays the button; the parent
/// button's `overflow: hidden` clips each ripple to the rounded
/// shape.
///
/// Note: an earlier revision used `position: absolute; size: 100%`
/// here, but that gave the element a near-zero height inside the
/// nested-absolute wrapper div, so the ripple painted as a thin
/// horizontal strip clipped to that strip's bounds. `inset: 0`
/// pins the element to the containing block on all four sides,
/// which is the "fill the parent" idiom in gpui.
struct RippleElement {
    state: Entity<RippleState>,
    /// Base ripple color (HSL). The alpha is multiplied by
    /// `(1 - t)` per paint, so this is the *peak* color.
    color: Hsla,
}

impl IntoElement for RippleElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for RippleElement {
    type RequestLayoutState = ();
    type PrepaintState = Vec<RipplePaint>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            position: Position::Absolute,
            inset: Edges::all(ZERO_LENGTH),
            ..Default::default()
        };
        // `inset: 0` on all four sides pins the element to the
        // padding box of its nearest positioned ancestor (the
        // button), with implicit size = parent's content size.
        // We don't set `size` — that's the whole point of the
        // inset pattern.
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // Read the current ripples out so the immutable borrow
        // is released before the cleanup `update` below.
        let ripples = self.state.read(cx).ripples.clone();

        if !ripples.is_empty() {
            // Ask the window to redraw on the next animation
            // frame. This is the gpui-idiomatic way to drive a
            // continuous animation: each frame's `prepaint`
            // re-requests the next frame while ripples are alive.
            window.request_animation_frame();
        }

        let now = Instant::now();
        let total = MD_RIPPLE_DURATION.as_secs_f32();
        let mut paints = Vec::with_capacity(ripples.len());

        for ripple in &ripples {
            let elapsed = now.duration_since(ripple.started).as_secs_f32();
            let t = (elapsed / total).clamp(0.0, 1.0);
            // ease-out cubic for the radius — fast start, slow
            // finish. Matches the MD spec's "decelerate" curve.
            let eased = 1.0 - (1.0 - t).powi(3);

            // Final radius = farthest distance from the click
            // point to any button corner, plus a small pad so
            // the ripple fully covers the corners at end of
            // animation. `bounds` and `ripple.center` are both
            // in window coords, so the math is direct.
            let corners = [
                (bounds.left(), bounds.top()),
                (bounds.right(), bounds.top()),
                (bounds.left(), bounds.bottom()),
                (bounds.right(), bounds.bottom()),
            ];
            let max_r = corners
                .iter()
                .map(|&(cx, cy)| {
                    let dx: f32 = (cx - ripple.center.x).into();
                    let dy: f32 = (cy - ripple.center.y).into();
                    (dx * dx + dy * dy).sqrt()
                })
                .fold(0.0_f32, f32::max)
                + MD_RIPPLE_CORNER_PADDING;

            let radius = max_r * eased;
            let radius_px = px(radius);
            // Alpha fades linearly from the peak to 0 as the
            // ripple grows. The color stays constant — it's the
            // alpha that gives the "ripple dissolving" effect.
            let alpha = MD_RIPPLE_PEAK_ALPHA * (1.0 - t);
            paints.push(RipplePaint {
                path: circle_path(point(ripple.center.x, ripple.center.y), radius_px),
                color: hsla(self.color.color.hue.into_degrees() / 360.0, self.color.saturation, self.color.lightness, alpha),
            });
        }

        // Drop any ripples that have finished animating.
        self.state.update(cx, |s, _cx| s.cleanup());

        paints
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        for paint in prepaint.drain(..) {
            // `paint_path` is the circular alternative to
            // `paint_quad`. The button's `overflow: hidden`
            // still clips each ripple to the rounded shape.
            window.paint_path(paint.path, paint.color);
        }
    }
}

/// Build a Material-Design-styled button with a click ripple
/// animation.
///
/// The button uses the MD2 "raised" visual: a flat primary-color
/// fill, 4px corners, 14px medium-weight label, white text. The
/// hover state lightens the fill (a stand-in for the M2
/// elevation bump, since gpui doesn't ship box shadows). The
/// ripple is a white 20%-alpha circle that expands from the
/// click point and fades to 0 over 450ms — the classic Material
/// ripple.
pub fn material_button(
    id: impl Into<ElementId>,
    label: SharedString,
    cx: &mut App,
    window: &mut Window,
) -> Stateful<Div> {
    let id_el = id.into();

    // `use_keyed_state` mints the ripple entity the first time
    // the button is rendered, and reuses it on every subsequent
    // render. It also installs an observer that re-renders the
    // current view when the entity is mutated — that's how a
    // click triggers a re-paint with the new ripple.
    let state: Entity<RippleState> =
        window.use_keyed_state(id_el.clone(), cx, |_window, _cx| RippleState::default());

    // M2 Raised Button palette: Teal 500.
    let bg = hsla(174.0 / 360.0, 1.0, 0.30, 1.0);
    let bg_hover = hsla(174.0 / 360.0, 1.0, 0.36, 1.0);
    let text_color = hsla(0.0, 0.0, 1.0, 1.0);
    let radius = px(4.0);

    // The RippleElement is a direct child of the button (the
    // button's `overflow: hidden` does the clipping). The
    // element positions itself with `position: absolute; inset:
    // 0` so it overlays the button exactly.
    let ripple_overlay = RippleElement {
        state: state.clone(),
        // White is the ripple base; alpha is applied per-paint.
        color: hsla(0.0, 0.0, 1.0, 1.0),
    };

    button(id_el.clone(), cx)
        // The headless button's on_click handler is the
        // button's semantic action. In this demo it's a no-op
        // — the ripple itself is the visible feedback. A real
        // app would pass the click callback through here.
        .on_click(|_ev, _window, _cx| {})
        .apply(
            div()
                .bg(bg)
                .rounded(radius)
                .cursor(gpui::CursorStyle::PointingHand)
                .relative()
                .overflow_hidden()
                .text_color(text_color)
                .text_size(px(14.))
                .font_weight(gpui::FontWeight(500.0))
                .px(px(16.))
                .py(px(8.))
                .min_h(px(36.))
                .on_mouse_down(MouseButton::Left, {
                    let state = state.clone();
                    move |event: &MouseDownEvent, _window, cx| {
                        // `event.position` is in window coords.
                        // We store it as-is and let `prepaint`
                        // do the (trivial) window-space math.
                        state.update(cx, |s, _cx| s.push(event.position));
                    }
                })
                .child(label)
                .child(ripple_overlay),
        )
        .hover(|s| s.bg(bg_hover))
}
