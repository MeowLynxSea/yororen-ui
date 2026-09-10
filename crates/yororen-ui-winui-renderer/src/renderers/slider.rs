//! `WinUISliderRenderer` — default `SliderRenderer` impl.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gpui::{
    App, BorderStyle, Bounds, Corners, CursorStyle, Edges, Element, ElementId, GlobalElementId,
    Hsla, InteractiveElement, IntoElement, LayoutId, MouseButton, PaintQuad, ParentElement, Path,
    PathBuilder, Pixels, Point, StatefulInteractiveElement, Style, Styled, Window, div, hsla,
    point, px, size,
};

use yororen_ui_core::headless::slider::SliderProps;
use yororen_ui_core::renderer::slider::{SliderRenderOutput, SliderRenderState, SliderRenderer};
use yororen_ui_core::theme::Theme;

use crate::animation::{
    TweenChannel, interaction_hovered, interaction_pressed, lerp_f32, lerp_hsla,
    set_interaction_hovered, set_interaction_pressed,
};

pub use yororen_ui_core::renderer::slider::SliderRenderer as SliderRendererTrait;

pub struct WinUISliderRenderer;

// Inherent helpers — *not* part of the `SliderRenderer` trait surface.
impl WinUISliderRenderer {
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
        theme
            .get_color("winui.subtle_fill")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default()
    }

    pub fn fill(&self, _state: &SliderRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.accent")
            .or_else(|| theme.get_color("action.primary.bg"))
            .unwrap_or_default()
    }

    pub fn knob(&self, _state: &SliderRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.accent")
            .or_else(|| theme.get_color("action.primary.bg"))
            .unwrap_or_default()
    }

    pub fn knob_outer(&self, _state: &SliderRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.slider_thumb_bg")
            .or_else(|| theme.get_color("winui.solid_bg"))
            .or_else(|| theme.get_color("surface.raised"))
            .unwrap_or_default()
    }

    /// Inner accent dot base diameter (12px in the reference),
    /// scaled 0.86 / 1.167 / 0.71 across rest / hover / press.
    pub fn inner_dot(&self, _state: &SliderRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.slider.inner_dot")
            .unwrap_or(12.0) as f32
    }

    /// Thumb drop shadow (`0 1px 3px rgba(0, 0, 0, 0.08)`).
    pub fn thumb_shadow(&self, _state: &SliderRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("shadow.slider_thumb")
            .unwrap_or_else(|| gpui::hsla(0., 0., 0., 0.08))
    }
}

impl SliderRenderer for WinUISliderRenderer {
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
        let knob_outer_bg = self.knob_outer(&state, theme);
        let knob_hover_bg = theme
            .get_color("winui.accent_hover")
            .or_else(|| theme.get_color("action.primary.hover_bg"))
            .unwrap_or(knob_bg);
        let knob_pressed_bg = theme
            .get_color("winui.accent_pressed")
            .or_else(|| theme.get_color("action.primary.active_bg"))
            .unwrap_or(knob_bg);

        let pct = ((props.value - props.min) / (props.max - props.min)).clamp(0.0, 1.0);

        let bounds_store: Arc<Mutex<Option<Bounds<Pixels>>>> = Arc::new(Mutex::new(None));

        let interaction_id = props.id.clone();
        let track_element = SliderTrackElement {
            element_id: (interaction_id.clone(), "slider-track-element").into(),
            interaction_id,
            bounds: bounds_store.clone(),
            pct,
            track_h,
            knob_size,
            track_bg,
            fill_bg,
            knob_bg,
            knob_outer_bg,
            knob_hover_bg,
            knob_pressed_bg,
            inner_dot: self.inner_dot(&state, theme),
            thumb_shadow: self.thumb_shadow(&state, theme),
            hover_progress: 0.0,
            pressed_progress: 0.0,
        };

        let mut visual: gpui::Stateful<gpui::Div> = div()
            .id(props.id.clone())
            .w(track_w)
            .h(px(32.0))
            .child(track_element);

        if props.disabled {
            visual = visual.cursor(CursorStyle::OperationNotAllowed);
        } else {
            visual = visual
                .on_hover({
                    let id = props.id.clone();
                    move |hovered, _win, cx| set_interaction_hovered(cx, id.clone(), *hovered)
                })
                .on_mouse_down(MouseButton::Left, {
                    let id = props.id.clone();
                    move |_, _win, cx| set_interaction_pressed(cx, id.clone(), true)
                })
                .on_mouse_up(MouseButton::Left, {
                    let id = props.id.clone();
                    move |_, _win, cx| set_interaction_pressed(cx, id.clone(), false)
                })
                .cursor(CursorStyle::PointingHand);
        }

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
    element_id: ElementId,
    interaction_id: ElementId,
    bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    pct: f32,
    track_h: f32,
    knob_size: f32,
    track_bg: Hsla,
    fill_bg: Hsla,
    knob_bg: Hsla,
    knob_outer_bg: Hsla,
    knob_hover_bg: Hsla,
    knob_pressed_bg: Hsla,
    inner_dot: f32,
    thumb_shadow: Hsla,
    hover_progress: f32,
    pressed_progress: f32,
}

/// Element-local animation state for the slider knob's inner accent
/// dot (scale + colour follow hover / press).
#[derive(Clone)]
struct SliderAnimState {
    hover: TweenChannel,
    pressed: TweenChannel,
}

impl Element for SliderTrackElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.element_id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let now = Instant::now();
        let hovered = interaction_hovered(cx, &self.interaction_id);
        let pressed = interaction_pressed(cx, &self.interaction_id);
        // WinUI state transition: 167ms `cubic-bezier(0, 0, 0, 1)`.
        let config = yororen_ui_core::animation::AnimationConfig::new()
            .with_duration(Duration::from_millis(167))
            .with_easing(crate::animation::fast_out_slow_in);

        let (hover_progress, pressed_progress, is_animating) = window.with_element_state(
            global_id.unwrap(),
            |state: Option<SliderAnimState>, _window| {
                let mut state = state.unwrap_or(SliderAnimState {
                    hover: TweenChannel::new(hovered, now),
                    pressed: TweenChannel::new(pressed, now),
                });
                let hover_animating = state.hover.update(hovered, &config, now);
                let pressed_animating = state.pressed.update(pressed, &config, now);
                (
                    (
                        state.hover.value(),
                        state.pressed.value(),
                        hover_animating || pressed_animating,
                    ),
                    state,
                )
            },
        );
        self.hover_progress = (config.easing)(hover_progress);
        self.pressed_progress = (config.easing)(pressed_progress);
        if is_animating {
            window.request_animation_frame();
        }

        let mut style = Style::default();
        style.size.width = gpui::relative(1.0).into();
        style.size.height = px(32.0).into();
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
        let track_y = bounds.top() + px((32.0 - self.track_h) / 2.0);
        let knob_y = bounds.top() + px((32.0 - self.knob_size) / 2.0);
        let track_w: f32 = bounds.size.width.into();
        let fill_w = px(self.pct * (track_w - self.knob_size));
        let knob_x = bounds.left() + px(self.pct * (track_w - self.knob_size));
        let track_radius = px(self.track_h / 2.0);

        // Track background — rounded on both ends (pill shape).
        let track_bounds = Bounds::new(
            point(bounds.left(), track_y),
            size(bounds.size.width, px(self.track_h)),
        );
        window.paint_quad(PaintQuad {
            bounds: track_bounds,
            corner_radii: Corners::all(track_radius).clamp_radii_for_quad_size(track_bounds.size),
            background: self.track_bg.into(),
            border_color: hsla(0., 0., 0., 0.),
            border_widths: Edges::default(),
            border_style: BorderStyle::default(),
        });

        // Fill — rounded on the left end only.
        let fill_bounds = Bounds::new(
            point(bounds.left(), track_y),
            size(fill_w, px(self.track_h)),
        );
        window.paint_quad(PaintQuad {
            bounds: fill_bounds,
            corner_radii: Corners {
                top_left: track_radius,
                top_right: px(0.),
                bottom_left: track_radius,
                bottom_right: px(0.),
            }
            .clamp_radii_for_quad_size(fill_bounds.size),
            background: self.fill_bg.into(),
            border_color: hsla(0., 0., 0., 0.),
            border_widths: Edges::default(),
            border_style: BorderStyle::default(),
        });

        // Knob — WinUI style: solid outer circle with an accent inner
        // dot that scales and recolours on hover / press.
        let knob_center = point(
            knob_x + px(self.knob_size / 2.0),
            knob_y + px(self.knob_size / 2.0),
        );
        let outer_radius = px(self.knob_size / 2.0);
        // Thumb drop shadow: 0 1px 3px rgba(0, 0, 0, 0.08).
        let knob_bounds = Bounds::new(
            point(knob_x, knob_y),
            size(px(self.knob_size), px(self.knob_size)),
        );
        window.paint_drop_shadows(
            knob_bounds,
            Corners::all(outer_radius),
            &[gpui::BoxShadow {
                color: self.thumb_shadow,
                offset: point(px(0.), px(1.)),
                blur_radius: px(3.),
                spread_radius: px(0.),
                inset: false,
            }],
        );
        let outer_path = circle_path(knob_center, outer_radius);
        window.paint_path(outer_path, self.knob_outer_bg);

        let base_inner = (self.inner_dot / 2.0).max(2.0);
        let mut scale = lerp_f32(0.86, 1.167, self.hover_progress);
        scale = lerp_f32(scale, 0.71, self.pressed_progress);
        let inner_radius = px(base_inner * scale);
        let inner_path = circle_path(knob_center, inner_radius);
        let mut dot_color = lerp_hsla(self.knob_bg, self.knob_hover_bg, self.hover_progress);
        if self.pressed_progress > 0.0 {
            dot_color = lerp_hsla(dot_color, self.knob_pressed_bg, self.pressed_progress);
        }
        window.paint_path(inner_path, dot_color);
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
