//! `TokenSkeletonRenderer` — default `SkeletonRenderer` impl.

use std::sync::{Arc, OnceLock};

use std::time::Instant;

use gpui::{
    AbsoluteLength, App, BorderStyle, Bounds, Corners, DefiniteLength, Div, Edges, Element,
    ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement, LayoutId, Length, Pixels,
    Position, Style, Styled, Window, div, hsla,
};

use yororen_ui_core::animation::ease_in_out;
use yororen_ui_core::headless::skeleton::SkeletonProps;
use yororen_ui_core::theme::Theme;

use gpui::ParentElement;

pub use yororen_ui_core::renderer::skeleton::{SkeletonRenderState, SkeletonRenderer};

pub struct TokenSkeletonRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenSkeletonRenderer {
    pub fn bg(&self, _state: &SkeletonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block {
            gpui::px(
                theme
                    .get_number("tokens.control.skeleton.block_min_h")
                    .unwrap_or(48.0) as f32,
            )
        } else {
            gpui::px(
                theme
                    .get_number("tokens.control.skeleton.line_h")
                    .unwrap_or(16.0) as f32,
            )
        }
    }
    pub fn border_radius(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block && !state.block_sharp {
            gpui::px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
        } else if !state.block {
            gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(9999.0) as f32)
        } else {
            gpui::px(0.0)
        }
    }
}

/// Pulse opacity range (matches `yororen_ui_core::animation::preset::defaults`).
const SKELETON_PULSE_MIN: f32 = 0.55;
const SKELETON_PULSE_MAX: f32 = 0.95;

/// Animation epoch — all skeletons in the app pulse in sync, which
/// is the standard loading-animation behavior. Captured once on
/// first paint via `OnceLock`.
static SKELETON_PULSE_EPOCH: OnceLock<Instant> = OnceLock::new();

/// A `Length` of zero pixels for `Edges::all` — pins the
/// absolutely-positioned overlay to all four sides of its parent
/// Div (the "fill the parent" idiom).
const ZERO_LENGTH: Length = Length::Definite(DefiniteLength::Absolute(AbsoluteLength::Pixels(
    gpui::px(0.),
)));

/// Custom `gpui::Element` that paints a single rounded quad with a
/// time-varying alpha, producing the skeleton pulse animation.
///
/// Sits inside a sized Div (which sets `min_h` + `rounded`). The
/// element positions itself `absolute inset 0` so it fills the
/// Div exactly; the Div's `rounded` style doesn't affect the
/// element's paint, so we re-apply the radius on the quad.
struct SkeletonPulseElement {
    bg: Hsla,
    radius: Pixels,
    duration_ms: u64,
}

impl IntoElement for SkeletonPulseElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SkeletonPulseElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

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
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        // Initialize the shared pulse epoch on first paint.
        let _ = SKELETON_PULSE_EPOCH.get_or_init(Instant::now);
        // Ask the window to redraw on the next animation frame —
        // this is what keeps the pulse moving.
        window.request_animation_frame();
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        // Phase = how far we are into one pulse cycle [0, 1).
        let epoch = SKELETON_PULSE_EPOCH.get_or_init(Instant::now);
        let elapsed_ms = epoch.elapsed().as_millis() as u64;
        let progress = if self.duration_ms == 0 {
            0.0
        } else {
            (elapsed_ms % self.duration_ms) as f32 / self.duration_ms as f32
        };
        // Shape `progress` into a triangle wave 0 → 1 → 0 so the
        // alpha ramps UP for the first half of the cycle, then
        // ramps BACK DOWN for the second half — a true "breath"
        // instead of a sawtooth that snaps from MAX back to MIN
        // at the cycle boundary.
        let tri = if progress < 0.5 {
            progress * 2.0
        } else {
            2.0 - progress * 2.0
        };
        // ease-in-out on each half: slow at the extremes (the
        // "hold" at the top and bottom of the breath), fast
        // through the middle.
        let eased = ease_in_out(tri);
        // Lerp alpha multiplier between min and max.
        let alpha_mult = SKELETON_PULSE_MIN + (SKELETON_PULSE_MAX - SKELETON_PULSE_MIN) * eased;
        let color = hsla(self.bg.color.hue.into_degrees() / 360.0, self.bg.saturation, self.bg.lightness, self.bg.alpha * alpha_mult);

        window.paint_quad(gpui::PaintQuad {
            bounds,
            // Clamp the corner radii to fit inside the quad's
            // bounds. gpui's `paint_quad` allows radii to exceed
            // the bounds, but that creates "sharp corners where
            // the circular arcs meet" — for a pill-shaped line
            // (h=12, radius=9999) the arcs intersect inside the
            // 12px space, leaving almost nothing visible. The
            // clamp gives us a clean pill at half the height.
            corner_radii: Corners::all(self.radius).clamp_radii_for_quad_size(bounds.size),
            background: color.into(),
            border_color: hsla(0., 0., 0., 0.),
            border_widths: Edges::default(),
            border_style: BorderStyle::default(),
        });
    }
}

impl SkeletonRenderer for TokenSkeletonRenderer {
    fn compose(&self, props: &SkeletonProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = SkeletonRenderState {
            block: props.block,
            block_sharp: props.block_sharp,
        };
        let bg = self.bg(&state, theme);
        let min_h = self.min_height(&state, theme);
        let radius = self.border_radius(&state, theme);
        // Pulse duration: theme `motion.duration_skeleton_pulse`
        // (1100ms default). The two `_1`/`_2` variants exist for
        // future staggered-shimmer support; we use the first.
        let duration_ms = theme
            .get_number("motion.duration_skeleton_pulse")
            .unwrap_or(1100.0) as u64;

        // The outer Div sets the size + radius. The pulse element
        // is `position: absolute; inset: 0` so it fills the Div
        // exactly. Explicit `w`/`h` props take precedence; for block
        // skeletons without an explicit height we fill the parent,
        // and for line skeletons without an explicit height we fall
        // back to the theme's line height.
        let mut el = div().rounded(radius);
        if let Some(w) = props.w {
            el = el.w(w);
        }
        if let Some(h) = props.h {
            el = el.h(h);
        } else if props.block {
            el = el.h_full();
        } else {
            el = el.min_h(min_h);
        }
        el.child(SkeletonPulseElement {
            bg,
            radius,
            duration_ms,
        })
    }
}

pub fn arc_skeleton<T: SkeletonRenderer + 'static>(r: T) -> Arc<dyn SkeletonRenderer> {
    Arc::new(r)
}
