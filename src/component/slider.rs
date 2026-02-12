use std::sync::Arc;

use gpui::{
    AppContext, Bounds, Div, Element, ElementId, Empty, GlobalElementId, Hsla, InspectorElementId,
    InteractiveElement, IntoElement, LayoutId, MouseButton, MouseDownEvent, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, px,
};

use gpui::prelude::FluentBuilder;

use crate::theme::ActiveTheme;

/// Creates a new slider element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn slider() -> Slider {
    Slider::new()
}

type ChangeFn = Arc<dyn Fn(f32, &mut gpui::Window, &mut gpui::App)>;

struct TrackBoundsElement {
    bounds_state: gpui::Entity<Bounds<gpui::Pixels>>,
    inner: gpui::AnyElement,
}

impl IntoElement for TrackBoundsElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TrackBoundsElement {
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
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        (self.inner.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        self.bounds_state.update(cx, |state, _| {
            *state = bounds;
        });
        self.inner.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        self.inner.paint(window, cx);
    }
}

#[derive(IntoElement)]
pub struct Slider {
    element_id: Option<ElementId>,
    base: Div,

    min: f32,
    max: f32,
    step: Option<f32>,
    value: Option<f32>,

    disabled: bool,

    height: Option<gpui::AbsoluteLength>,
    bg_color: Option<Hsla>,
    fill_color: Option<Hsla>,
    border_color: Option<Hsla>,
    focus_border_color: Option<Hsla>,

    on_change: Option<ChangeFn>,
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

impl Slider {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: gpui::div(),

            min: 0.0,
            max: 1.0,
            step: None,
            value: None,

            disabled: false,

            height: None,
            bg_color: None,
            fill_color: None,
            border_color: None,
            focus_border_color: None,

            on_change: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        assert!(
            min < max,
            "Slider range: min ({min}) must be less than max ({max})"
        );
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        assert!(step > 0.0, "Slider step must be greater than 0");
        self.step = Some(step);
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = Some(value);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn height(mut self, height: gpui::AbsoluteLength) -> Self {
        self.height = Some(height);
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg_color = Some(color.into());
        self
    }

    pub fn fill(mut self, color: impl Into<Hsla>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn focus_border(mut self, color: impl Into<Hsla>) -> Self {
        self.focus_border_color = Some(color.into());
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(f32, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl ParentElement for Slider {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Slider {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Slider {}

impl RenderOnce for Slider {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let element_id = self.element_id;

        let id = element_id.expect(
            "Slider requires an id for internal state management. Use `.id()` or `.key()` to set an id.",
        );

        let disabled = self.disabled;
        let theme = cx.theme().clone();
        let height = self.height.unwrap_or_else(|| px(36.).into());

        // Slider has no outer container background; `bg_color` controls the track color instead.
        let track_bg = if disabled {
            theme.surface.sunken
        } else {
            self.bg_color.unwrap_or(theme.surface.sunken)
        };

        let fill = if disabled {
            theme.content.disabled
        } else {
            self.fill_color.unwrap_or(theme.action.primary.bg)
        };

        let min = self.min;
        let max = self.max;
        let step = self.step;

        let on_change = self.on_change;
        let use_internal_value = on_change.is_none();
        let initial_value = self.value.unwrap_or(min);
        let internal_value = if use_internal_value {
            Some(window.use_keyed_state((id.clone(), "ui:slider:value"), cx, |_, _| initial_value))
        } else {
            None
        };

        let mut value = if use_internal_value {
            *internal_value
                .as_ref()
                .expect("internal value should exist")
                .read(cx)
        } else {
            self.value.unwrap_or(min)
        };

        value = clamp(value, min.min(max), max.max(min));
        let t = if (max - min).abs() <= f32::EPSILON {
            0.0
        } else {
            clamp((value - min) / (max - min), 0.0, 1.0)
        };

        let knob_diameter = 16.0;
        let track_height = 6.0;

        let track_bounds_state =
            window.use_keyed_state((id.clone(), "ui:slider:track-bounds"), cx, |_, _| {
                Bounds::default()
            });

        let set_from_mouse_x = {
            let internal_value = internal_value.clone();
            let on_change = on_change.clone();
            move |x: f32,
                  bounds: Bounds<gpui::Pixels>,
                  window: &mut gpui::Window,
                  cx: &mut gpui::App| {
                if bounds.size.width <= px(1.) {
                    return;
                }
                let left: f32 = bounds.left().into();
                let width: f32 = bounds.size.width.into();
                let mut ratio = (x - left) / width;
                ratio = clamp(ratio, 0.0, 1.0);

                let mut new_value = min + (max - min) * ratio;
                if let Some(step) = step.filter(|s| *s > 0.0) {
                    new_value = quantize(new_value, min, step);
                }
                new_value = clamp(new_value, min.min(max), max.max(min));

                if let Some(internal_value) = &internal_value {
                    internal_value.update(cx, |state, cx| {
                        *state = new_value;
                        cx.notify();
                    });
                }
                if let Some(handler) = &on_change {
                    handler(new_value, window, cx);
                }
            }
        };

        let mut base = self
            .base
            .id(id.clone())
            .h(height)
            .w_full()
            .flex()
            .items_center()
            .px_3();

        base = if disabled {
            base.opacity(0.6).cursor_not_allowed()
        } else {
            base.cursor_pointer()
        };

        // Make the interaction hitbox more lenient: clicking or dragging anywhere in the slider's
        // container adjusts the value based on the track's bounds.
        base = base
            .on_drag((), move |_v: &(), _pos, _window, cx| cx.new(|_| Empty))
            .on_mouse_down(MouseButton::Left, {
                let track_bounds_state = track_bounds_state.clone();
                let set_from_mouse_x = set_from_mouse_x.clone();
                move |ev: &MouseDownEvent, window, cx| {
                    if disabled {
                        return;
                    }

                    let bounds = *track_bounds_state.read(cx);
                    if bounds.size.width > px(1.) {
                        let x: f32 = ev.position.x.into();
                        set_from_mouse_x(x, bounds, window, cx);
                    }

                    window.refresh();
                }
            })
            .on_drag_move::<()>({
                let track_bounds_state = track_bounds_state.clone();
                let set_from_mouse_x = set_from_mouse_x.clone();
                move |ev, window, cx| {
                    if disabled {
                        return;
                    }

                    let bounds = *track_bounds_state.read(cx);
                    if bounds.size.width > px(1.) {
                        let x: f32 = ev.event.position.x.into();
                        set_from_mouse_x(x, bounds, window, cx);
                    }
                }
            });

        base.child(TrackBoundsElement {
            bounds_state: track_bounds_state.clone(),
            inner: gpui::div()
                .id((id.clone(), "ui:slider:track"))
                .relative()
                .w_full()
                .h(px(track_height))
                .rounded_full()
                .bg(track_bg)
                .when(!disabled, |this| this.cursor_pointer())
                .on_drag((), move |_v: &(), _pos, _window, cx| cx.new(|_| Empty))
                .on_mouse_down(MouseButton::Left, {
                    let track_bounds_state = track_bounds_state.clone();
                    let set_from_mouse_x = set_from_mouse_x.clone();
                    move |ev: &MouseDownEvent, window, cx| {
                        if disabled {
                            return;
                        }

                        let bounds = *track_bounds_state.read(cx);
                        if bounds.size.width > px(1.) {
                            let x: f32 = ev.position.x.into();
                            set_from_mouse_x(x, bounds, window, cx);
                        }

                        window.refresh();
                    }
                })
                .on_drag_move::<()>({
                    let set_from_mouse_x = set_from_mouse_x.clone();
                    move |ev, window, cx| {
                        if disabled {
                            return;
                        }
                        let x: f32 = ev.event.position.x.into();
                        set_from_mouse_x(x, ev.bounds, window, cx);
                    }
                })
                .child(
                    gpui::div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .h(px(track_height))
                        .rounded_full()
                        .bg(fill)
                        .w(gpui::relative(t)),
                )
                .child(
                    gpui::div()
                        .absolute()
                        .top(px(-(knob_diameter - track_height) / 2.0))
                        .left_0()
                        .h(px(knob_diameter))
                        .w(gpui::relative(t))
                        .flex()
                        .items_center()
                        .justify_end()
                        .child(
                            gpui::div()
                                .w(px(knob_diameter))
                                .h(px(knob_diameter))
                                .rounded_full()
                                .bg(theme.action.primary.bg)
                                .hover(|this| this.bg(theme.action.primary.hover_bg))
                                .border_1()
                                .border_color(theme.surface.raised),
                        ),
                )
                .into_any_element(),
        })
    }
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}

fn quantize(value: f32, origin: f32, step: f32) -> f32 {
    let n = ((value - origin) / step).round();
    origin + n * step
}
