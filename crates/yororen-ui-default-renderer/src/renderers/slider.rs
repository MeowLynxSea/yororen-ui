//! `TokenSliderRenderer` — default `SliderRenderer` impl.

use std::sync::{Arc, Mutex};

use gpui::{
    App, Bounds, Corners, CursorStyle, Element, GlobalElementId, Hsla, InteractiveElement,
    IntoElement, LayoutId, PaintQuad, ParentElement, Path, PathBuilder, Pixels, Point, Style,
    Styled, Window, div, point, px, size,
};

use yororen_ui_core::headless::slider::SliderProps;
use yororen_ui_core::renderer::slider::{SliderRenderOutput, SliderRenderState, SliderRenderer};
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::slider::SliderRenderer as SliderRendererTrait;

pub struct TokenSliderRenderer;

// Inherent helpers — *not* part of the `SliderRenderer` trait surface.
impl TokenSliderRenderer {
    pub fn track_h(&self, _state: &SliderRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.slider.track_h")
            .unwrap_or(6.0) as f32
    }

    pub fn knob_size(&self, _state: &SliderRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.slider.thumb_size")
            .unwrap_or(16.0) as f32
    }

    pub fn hit_padding(&self, _state: &SliderRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.slider.hit_padding")
            .unwrap_or(8.0) as f32
    }

    pub fn track_w(&self, _state: &SliderRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.slider.track_w")
                .unwrap_or(240.0) as f32,
        )
    }

    pub fn track(&self, _state: &SliderRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }

    pub fn fill(&self, _state: &SliderRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }

    pub fn knob(&self, _state: &SliderRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
}

impl SliderRenderer for TokenSliderRenderer {
    fn compose(&self, props: &SliderProps, cx: &App) -> SliderRenderOutput {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = SliderRenderState {
            disabled: props.disabled,
        };

        let track_h = self.track_h(&state, theme);
        let knob_size = self.knob_size(&state, theme);
        let track_w = self.track_w(&state, theme);
        let track_bg = self.track(&state, theme);
        let fill_bg = self.fill(&state, theme);
        let knob_bg = self.knob(&state, theme);

        let pct = ((props.value - props.min) / (props.max - props.min)).clamp(0.0, 1.0);

        let bounds_store: Arc<Mutex<Option<Bounds<Pixels>>>> = Arc::new(Mutex::new(None));

        let track_element = SliderTrackElement {
            bounds: bounds_store.clone(),
            pct,
            track_h,
            knob_size,
            track_bg,
            fill_bg,
            knob_bg,
        };

        let visual = div()
            .id(props.id.clone())
            .w(track_w)
            .h(px(24.0))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .child(track_element);

        SliderRenderOutput {
            visual,
            track_bounds: bounds_store,
        }
    }
}

/// Internal `Element` that paints the slider track, fill and knob.
/// Its only job besides painting is to store its laid-out `bounds`
/// in the shared `Arc<Mutex<...>>` during `prepaint` so the
/// headless layer can convert window-relative mouse positions to
/// local coordinates.
struct SliderTrackElement {
    bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    pct: f32,
    track_h: f32,
    knob_size: f32,
    track_bg: Hsla,
    fill_bg: Hsla,
    knob_bg: Hsla,
}

impl Element for SliderTrackElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.0).into();
        style.size.height = px(24.0).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        *self.bounds.lock().unwrap() = Some(bounds);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let track_y = bounds.top() + px((24.0 - self.track_h) / 2.0);
        let knob_y = bounds.top() + px((24.0 - self.knob_size) / 2.0);
        let track_w: f32 = bounds.size.width.into();
        let fill_w = px(self.pct * (track_w - self.knob_size));
        let knob_x = bounds.left() + px(self.pct * (track_w - self.knob_size));
        let track_radius = px(self.track_h / 2.0);

        // Track background — rounded on both ends (pill shape).
        let track_bounds = Bounds::new(
            point(bounds.left(), track_y),
            size(bounds.size.width, px(self.track_h)),
        );
        window.paint_quad(
            PaintQuad::new(track_bounds)
                .corner_radii(
                    Corners::all(track_radius).clamp_radii_for_quad_size(track_bounds.size),
                )
                .background(self.track_bg),
        );

        // Fill — rounded on the left end only.
        let fill_bounds = Bounds::new(
            point(bounds.left(), track_y),
            size(fill_w, px(self.track_h)),
        );
        window.paint_quad(
            PaintQuad::new(fill_bounds)
                .corner_radii(
                    Corners {
                        top_left: track_radius,
                        top_right: px(0.),
                        bottom_left: track_radius,
                        bottom_right: px(0.),
                    }
                    .clamp_radii_for_quad_size(fill_bounds.size),
                )
                .background(self.fill_bg),
        );

        // Knob — circular.
        let knob_center = point(
            knob_x + px(self.knob_size / 2.0),
            knob_y + px(self.knob_size / 2.0),
        );
        let knob_path = circle_path(knob_center, px(self.knob_size / 2.0));
        window.paint_path(knob_path, self.knob_bg);
    }
}

/// Build a closed circular `Path<Pixels>` of the given `radius`
/// around `center`.
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

impl IntoElement for SliderTrackElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}
