//! `WinUIProgressBarRenderer` — default `ProgressBarRenderer` impl.
//!
//! Determinate mode paints a 3px indicator (radius 1.5px) on a 1px
//! track. Indeterminate mode hides the track and loops two sliding
//! segments — 40% and 60% of the track width — using the exact
//! keyframes from the reference `WinProgressBar`
//! (`win-progress-bar-indeterminate` / `-2`, 2s infinite).

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use gpui::{
    App, Div, Element, ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement, LayoutId,
    ParentElement, Pixels, Styled, Window, div, px,
};

use yororen_ui_core::headless::progress::ProgressBarProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::progress::{ProgressBarRenderState, ProgressBarRenderer};

pub struct WinUIProgressBarRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIProgressBarRenderer {
    pub fn track(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.subtle_fill")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default()
    }

    pub fn fill(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.accent")
            .or_else(|| theme.get_color("action.primary.bg"))
            .unwrap_or_default()
    }

    /// Indicator height: 3px in the reference.
    pub fn height(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.progress.bar_default_h")
                .unwrap_or(3.0) as f32,
        )
    }

    /// Track height: 1px in the reference (the indicator floats on
    /// top of a hairline track).
    pub fn track_height(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.progress.track_h")
                .unwrap_or(1.0) as f32,
        )
    }

    /// Indicator corner radius: half the indicator height (1.5px).
    pub fn border_radius(&self, state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        let configured = theme
            .get_number("tokens.control.progress.bar_radius")
            .unwrap_or(0.0) as f32;
        if configured > 0.0 {
            gpui::px(configured)
        } else {
            let h: f32 = self.height(state, theme).into();
            gpui::px(h / 2.0)
        }
    }

    /// Full indeterminate loop duration (2s in the reference).
    pub fn indeterminate_duration_ms(&self, _state: &ProgressBarRenderState, theme: &Theme) -> u64 {
        theme
            .get_number("tokens.control.progress.indeterminate_duration")
            .or_else(|| theme.get_number("tokens.motion.duration_progress_bar"))
            .unwrap_or(2000.0) as u64
    }
}

impl ProgressBarRenderer for WinUIProgressBarRenderer {
    fn compose(&self, props: &ProgressBarProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ProgressBarRenderState {
            indeterminate: props.indeterminate,
            has_custom_height: props.has_custom_height,
        };
        let track = self.track(&state, theme);
        let fill = self.fill(&state, theme);
        let h = self.height(&state, theme);
        let r = self.border_radius(&state, theme);

        if props.indeterminate {
            return div().w_full().h(h).child(IndeterminateBarElement {
                id: (props.id.clone(), "indeterminate").into(),
                track,
                fill,
                height: h,
                radius: r,
                duration_ms: self.indeterminate_duration_ms(&state, theme),
            });
        }

        // Compute fill ratio, clamped to [0, 1].
        let ratio = if props.max <= 0.0 {
            0.0
        } else {
            (props.value / props.max).clamp(0.0, 1.0)
        };
        // Reference determinate layout: a 1px hairline track
        // centred inside the bar height, with the 3px indicator
        // overlaid on top.
        let track_h_px = self.track_height(&state, theme);
        div()
            .relative()
            .w_full()
            .h(h)
            .child(
                // 1px hairline, vertically centred.
                div()
                    .absolute()
                    .top((h - track_h_px) / 2.0)
                    .left_0()
                    .right_0()
                    .h(track_h_px)
                    .bg(track)
                    .rounded(px(0.5)),
            )
            .child(
                // 3px indicator overlay.
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .h(h)
                    .w(gpui::relative(ratio))
                    .bg(fill)
                    .rounded(r),
            )
    }
}

/// Process-wide epoch for the indeterminate loop. The bar element
/// is rebuilt on every frame (compose runs per render), so a
/// per-element start time would reset each frame and freeze the
/// animation at phase 0 — the same reason the skeleton pulse uses a
/// shared `OnceLock` epoch.
static INDETERMINATE_EPOCH: OnceLock<Instant> = OnceLock::new();

/// Reference keyframes, expressed as translateX in units of the
/// segment's own width over one 2s loop:
///
/// - Segment 1 (40% wide): 0% → -100%, 75% → 300%, then holds.
/// - Segment 2 (60% wide): 0% → -150% (holds through 37.5%), then
///   → 166% at 100%.
fn segment1_offset(p: f32) -> f32 {
    if p < 0.75 {
        let t = p / 0.75;
        -1.0 + t * 4.0
    } else {
        3.0
    }
}

fn segment2_offset(p: f32) -> f32 {
    if p < 0.375 {
        -1.5
    } else {
        let t = (p - 0.375) / 0.625;
        -1.5 + t * 3.16
    }
}

/// Custom element painting the two sliding indeterminate segments
/// (and hiding the track, matching the reference's
/// `.state-Indeterminate .ProgressBarTrack { opacity: 0 }`).
struct IndeterminateBarElement {
    id: ElementId,
    track: Hsla,
    fill: Hsla,
    height: Pixels,
    radius: Pixels,
    duration_ms: u64,
}

impl Element for IndeterminateBarElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // Keep the loop alive every frame while mounted.
        window.request_animation_frame();

        let mut style = gpui::Style::default();
        style.size.width = gpui::relative(1.0).into();
        style.size.height = self.height.into();
        (window.request_layout(style, [], _cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: gpui::Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        use gpui::{Bounds, Corners, PaintQuad, point, size};
        let track_w: f32 = bounds.size.width.into();
        let h: f32 = self.height.into();
        let r = self.radius;
        let _ = self.track; // track is hidden in indeterminate mode
        let epoch = INDETERMINATE_EPOCH.get_or_init(Instant::now);
        let elapsed = epoch.elapsed().as_millis() as f64;
        let p = if self.duration_ms > 0 {
            ((elapsed % self.duration_ms as f64) / self.duration_ms as f64) as f32
        } else {
            0.0
        };

        // Segments travel from fully off-screen left to fully
        // off-screen right; clip to the track bounds so the
        // out-of-band portions never paint over neighbours (the
        // reference relies on CSS `overflow: hidden`).
        window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
            let segments = [(0.4, segment1_offset(p)), (0.6, segment2_offset(p))];
            for (width_ratio, offset) in segments {
                let seg_w = track_w * width_ratio;
                let seg_x = offset * seg_w;
                let seg_bounds = Bounds::new(
                    point(bounds.left() + px(seg_x), bounds.top()),
                    size(px(seg_w), px(h)),
                );
                window.paint_quad(PaintQuad {
                    bounds: seg_bounds,
                    corner_radii: Corners::all(r).clamp_radii_for_quad_size(seg_bounds.size),
                    background: self.fill.into(),
                    border_color: gpui::black().opacity(0.),
                    border_widths: gpui::Edges::default(),
                    border_style: gpui::BorderStyle::default(),
                });
            }
        });
    }
}

impl IntoElement for IndeterminateBarElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Duration helper re-exported for callers that want the reference
/// indeterminate loop length without reading the theme directly.
pub fn indeterminate_loop_duration(theme: &Theme) -> Duration {
    Duration::from_millis(
        theme
            .get_number("tokens.control.progress.indeterminate_duration")
            .or_else(|| theme.get_number("tokens.motion.duration_progress_bar"))
            .unwrap_or(2000.0) as u64,
    )
}

pub fn arc_progress_bar<T: ProgressBarRenderer + 'static>(r: T) -> Arc<dyn ProgressBarRenderer> {
    Arc::new(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/winui-dark.json");
        Theme::from_json(json).expect("winui-dark.json is valid")
    }

    #[test]
    fn indeterminate_keyframes_match_reference() {
        // Segment 1: -100% at 0%, 300% at 75%, holds through 100%.
        assert_eq!(segment1_offset(0.0), -1.0);
        assert_eq!(segment1_offset(0.375), -1.0 + 0.5 * 4.0);
        assert_eq!(segment1_offset(0.75), 3.0);
        assert_eq!(segment1_offset(1.0), 3.0);

        // Segment 2: -150% until 37.5%, then travels to 166%.
        assert_eq!(segment2_offset(0.0), -1.5);
        assert_eq!(segment2_offset(0.375), -1.5);
        assert!((segment2_offset(1.0) - 1.66).abs() < 0.001);
    }

    #[test]
    fn geometry_reads_reference_tokens() {
        let theme = fixture();
        let r = WinUIProgressBarRenderer;
        let state = ProgressBarRenderState::default();
        let h: f32 = r.height(&state, &theme).into();
        assert_eq!(h, 3.0);
        let track_h: f32 = r.track_height(&state, &theme).into();
        assert_eq!(track_h, 1.0);
        let radius: f32 = r.border_radius(&state, &theme).into();
        assert_eq!(radius, 1.5);
        assert_eq!(
            r.indeterminate_duration_ms(&state, &theme),
            theme
                .get_number("tokens.control.progress.indeterminate_duration")
                .unwrap() as u64
        );
    }
}
