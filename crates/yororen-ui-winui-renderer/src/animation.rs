//! Renderer-side animation helpers.
//!
//! These types live in the renderer crate because they deal with
//! pixels, transforms, and `gpui::Element` — all visual concerns.
//! The data layer (`AnimatedVisibility`) stays in
//! `yororen_ui_core::animation`.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, Div, Element, ElementId, Entity, Global, GlobalElementId, Hsla,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, ParentElement, Pixels, Rgba,
    Stateful, Styled, Window, div, px,
};
use yororen_ui_core::animation::{AnimatedPresenceState, AnimationConfig, SlideDirection};

/// Renderer-scoped interaction state shared by WinUI controls.
///
/// GPUI has no CSS-style `transition`; hover/pressed styles are either
/// applied instantly via `.hover(...)`/`.active(...)` or must be driven
/// by the element tree itself. To animate between interaction states we
/// record which element ids are hovered / pressed here (from
/// `.on_hover` / `.on_mouse_down` / `.on_mouse_up` listeners) and let
/// custom elements (see [`AnimatedStateElement`]) read the flags each
/// frame while they tween their own progress.
#[derive(Default)]
pub struct WinUIInteractions {
    hovered: HashSet<ElementId>,
    pressed: HashSet<ElementId>,
}

impl WinUIInteractions {
    pub fn set_hovered(&mut self, id: ElementId, hovered: bool) {
        if hovered {
            self.hovered.insert(id);
        } else {
            self.hovered.remove(&id);
        }
    }

    pub fn set_pressed(&mut self, id: ElementId, pressed: bool) {
        if pressed {
            self.pressed.insert(id);
        } else {
            self.pressed.remove(&id);
        }
    }

    pub fn is_hovered(&self, id: &ElementId) -> bool {
        self.hovered.contains(id)
    }

    pub fn is_pressed(&self, id: &ElementId) -> bool {
        self.pressed.contains(id)
    }
}

impl Global for WinUIInteractions {}

/// Mark an element as hovered / no-longer-hovered. Called from
/// `.on_hover` listeners on renderer shells.
pub fn set_interaction_hovered(cx: &mut App, id: ElementId, hovered: bool) {
    cx.default_global::<WinUIInteractions>()
        .set_hovered(id, hovered);
}

/// Mark an element as pressed / released. Called from
/// `.on_mouse_down` / `.on_mouse_up` listeners.
pub fn set_interaction_pressed(cx: &mut App, id: ElementId, pressed: bool) {
    cx.default_global::<WinUIInteractions>()
        .set_pressed(id, pressed);
}

/// Whether `id` is currently hovered (only meaningful while a
/// `WinUIInteractions` global exists, i.e. after the first listener
/// fired).
pub fn interaction_hovered(cx: &App, id: &ElementId) -> bool {
    cx.try_global::<WinUIInteractions>()
        .map(|s| s.is_hovered(id))
        .unwrap_or(false)
}

/// Whether `id` is currently pressed.
pub fn interaction_pressed(cx: &App, id: &ElementId) -> bool {
    cx.try_global::<WinUIInteractions>()
        .map(|s| s.is_pressed(id))
        .unwrap_or(false)
}

/// Linear interpolation in RGBA space (avoids hue-wrapping artifacts
/// that plain HSLA lerping would produce).
pub(crate) fn lerp_hsla(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0.0, 1.0);
    let a = Rgba::from(a);
    let b = Rgba::from(b);
    Hsla::from(Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    })
}

/// Linear interpolation for pixel-ish scalars.
pub(crate) fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
}

/// Wrap a fully-assembled text-ish input (`Stateful<Div>`, keymap and
/// children already wired) in an [`AnimatedStateElement`] that draws
/// the WinUI Fluent TextBox look:
///
/// - top / left / right: 1px thin `side_border`
/// - bottom: 2px underline (`bottom_rest` → `bottom_focused`)
/// - background rest → hover: `bg_rest` → `bg_hover`
/// - focused background is `bg_focused` and never reacts to hover
#[allow(clippy::too_many_arguments)]
pub fn animated_input_border(
    keyed: Stateful<Div>,
    id: impl Into<ElementId>,
    hover_id: impl Into<ElementId>,
    side_border: Hsla,
    bottom_rest: Hsla,
    bottom_focused: Hsla,
    bg_rest: Hsla,
    bg_hover: Hsla,
    bg_focused: Hsla,
    focused: bool,
) -> AnyElement {
    let id: ElementId = id.into();
    let hover_id: ElementId = hover_id.into();

    // Static bits: three thin sides on the wrapper + a 2px bottom
    // underline child. Border colours do NOT change on hover.
    let bottom = if focused { bottom_focused } else { bottom_rest };
    let underline = div()
        .absolute()
        .left_0()
        .right_0()
        .bottom_0()
        .h(px(2.0))
        .bg(bottom);
    let keyed = keyed
        .bg(if focused { bg_focused } else { bg_rest })
        .border_color(side_border);
    let mut keyed = keyed.child(underline);
    {
        let style = keyed.style();
        style.border_widths = gpui::EdgesRefinement {
            top: Some(px(1.0).into()),
            right: Some(px(1.0).into()),
            bottom: Some(px(0.0).into()),
            left: Some(px(1.0).into()),
        };
    }

    // `bg_hover` is usually a translucent overlay (e.g.
    // `winui.ctrl_fill_hover`). Lerping RGBA directly between an
    // opaque rest and a semi-transparent hover produces a
    // semi-transparent bright middle colour that composits much
    // lighter than either endpoint. Pre-composite the hover onto the
    // rest background so the animation stays between two opaque,
    // monotonic colours.
    let hover_bg = bg_rest.blend(bg_hover);
    let wrapper = AnimatedStateElement::new(
        (id.clone(), "input-border"),
        hover_id,
        false,
        keyed,
        AnimationConfig::default().with_duration(Duration::from_millis(150)),
        move |d: Stateful<Div>, hover, _pressed, _checked| {
            // Only the background animates: inactive hover lightens,
            // focused uses a darker fixed background with no hover.
            let bg = if focused {
                bg_focused
            } else {
                lerp_hsla(bg_rest, hover_bg, hover)
            };
            d.bg(bg)
        },
    );
    wrapper.into_any_element()
}

/// A custom element that keeps a child mounted while it exits and
/// applies a fade + slide animation driven by an
/// [`AnimatedVisibility`] held in a `gpui::Entity<S>`.
///
/// This is the renderer-side half of the presence animation system.
/// The headless state owns *when* to open/close; this element owns
/// *how it looks* while doing so.
pub struct AnimatedPresenceElement<S: AnimatedPresenceState> {
    pub state: Entity<S>,
    pub id: ElementId,
    /// Direction the child slides *from* while entering (and *to*
    /// while exiting).
    pub direction: SlideDirection,
    pub distance: Pixels,
    child: Option<Div>,
}

impl<S: AnimatedPresenceState> AnimatedPresenceElement<S> {
    pub fn new(
        state: Entity<S>,
        id: impl Into<ElementId>,
        direction: SlideDirection,
        distance: Pixels,
        child: Div,
    ) -> Self {
        Self {
            state,
            id: id.into(),
            direction,
            distance,
            child: Some(child),
        }
    }
}

impl<S: AnimatedPresenceState> IntoElement for AnimatedPresenceElement<S> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[derive(Clone)]
struct PresenceElementState {
    start: Instant,
    previous_target: bool,
    previous_progress: f32,
}

impl<S: AnimatedPresenceState> Element for AnimatedPresenceElement<S> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let now = Instant::now();

        // Read or initialize per-element animation bookkeeping.
        let (mut el_state, _) = window.with_element_state(
            global_id.unwrap(),
            |state: Option<PresenceElementState>, _window| {
                let state = state.unwrap_or(PresenceElementState {
                    start: now,
                    previous_target: false,
                    previous_progress: 0.0,
                });
                ((state.clone(), ()), state)
            },
        );

        // Read/update the shared visibility state. We update it here
        // so the entity always reflects the current progress and the
        // app can query `is_visible()` to decide whether to keep the
        // overlay mounted.
        let (progress, target, _is_animating, enter_config, exit_config) =
            self.state.update(cx, |s, _cx| {
                let v = s.visibility_mut();
                let target = v.target;

                if el_state.previous_target != target {
                    // Target changed since last frame: reset the
                    // element-local animation clock, but keep the
                    // current progress so the transition is smooth.
                    el_state.start = now;
                    el_state.previous_target = target;
                }

                let dt = el_state.start.elapsed();
                v.update(dt);

                (
                    v.progress,
                    v.target,
                    v.is_animating(),
                    v.enter_config.clone(),
                    v.exit_config.clone(),
                )
            });

        // Request another frame whenever progress changed. This keeps
        // the animation running while it is active, and schedules one
        // final frame after it reaches a boundary (e.g. progress == 0)
        // so the parent can re-read `is_visible()` and unmount.
        if el_state.previous_progress != progress {
            window.request_animation_frame();
            el_state.previous_progress = progress;
        }

        // Save the element-local clock back.
        window.with_element_state(global_id.unwrap(), |_state, _window| ((), el_state));

        let config = if target { enter_config } else { exit_config };
        let eased = (config.easing)(progress);

        // Apply fade + slide transform based on the current phase.
        let distance_f: f32 = self.distance.into();
        let translate = distance_f * (1.0 - eased);

        let child = self
            .child
            .take()
            .expect("AnimatedPresenceElement::request_layout called once");
        let mut styled = child.id(self.id.clone()).opacity(progress);

        match self.direction {
            SlideDirection::Left => styled = styled.ml(px(-translate)),
            SlideDirection::Right => styled = styled.ml(px(translate)),
            SlideDirection::Up => styled = styled.mt(px(-translate)),
            SlideDirection::Down => styled = styled.mt(px(translate)),
        }

        let mut element = styled.into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// Apply a one-shot fade-in animation to a `Div`.
///
/// This is the simplest animation path: the element mounts, fades in,
/// and stays at full opacity. It is used by components whose state is
/// not entity-backed (e.g. the `Overlay` scrim when the caller mounts
/// it conditionally).
pub fn fade_in_on_mount(
    el: Div,
    id: impl Into<ElementId>,
    duration: Duration,
    easing: fn(f32) -> f32,
) -> gpui::AnimationElement<Div> {
    use gpui::AnimationExt;
    let animation = gpui::Animation::new(duration).with_easing(easing);
    el.with_animation(id, animation, move |this, progress| {
        let eased = easing(progress);
        this.opacity(eased)
    })
}

// =====================================================================
// Boolean transition elements — used by toggle controls (switch,
// checkbox) whose state is a plain `bool` rather than an entity-backed
// `AnimatedVisibility`.
// =====================================================================

#[derive(Clone)]
struct BooleanElementState {
    start: Instant,
    previous_value: bool,
    previous_progress: f32,
}

impl BooleanElementState {
    fn progress(&self, value: bool, config: &AnimationConfig) -> f32 {
        let duration_secs = config.duration.as_secs_f32();
        let rate = if duration_secs > 0.0 {
            self.start.elapsed().as_secs_f32() / duration_secs
        } else {
            1.0
        };
        if value {
            (self.previous_progress + rate).min(1.0)
        } else {
            (self.previous_progress - rate).max(0.0)
        }
    }
}

/// A custom element that animates a child's opacity based on a
/// boolean value.
///
/// When `value` is `true` the child fades in; when `value` is `false`
/// it fades out. The element keeps its own element-local animation
/// clock so the transition is smooth even though the underlying state
/// is not entity-backed.
pub struct AnimatedOpacityElement {
    pub id: ElementId,
    pub value: bool,
    child: Option<Div>,
    pub config: AnimationConfig,
}

impl AnimatedOpacityElement {
    pub fn new(id: impl Into<ElementId>, value: bool, child: Div) -> Self {
        Self {
            id: id.into(),
            value,
            child: Some(child),
            config: AnimationConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }
}

impl IntoElement for AnimatedOpacityElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for AnimatedOpacityElement {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let now = Instant::now();
        let (progress, is_animating) = window.with_element_state(
            global_id.unwrap(),
            |state: Option<BooleanElementState>, _window| {
                let mut state = state.unwrap_or(BooleanElementState {
                    start: now,
                    previous_value: self.value,
                    previous_progress: if self.value { 1.0 } else { 0.0 },
                });
                if state.previous_value != self.value {
                    state.start = now;
                    state.previous_value = self.value;
                }
                let progress = state.progress(self.value, &self.config);
                let is_animating =
                    (self.value && progress < 1.0) || (!self.value && progress > 0.0);
                state.previous_progress = progress;
                ((progress, is_animating), state)
            },
        );

        if is_animating {
            window.request_animation_frame();
        }

        let eased = (self.config.easing)(progress);
        let child = self
            .child
            .take()
            .expect("AnimatedOpacityElement::request_layout called once");
        let mut styled = child.id(self.id.clone()).opacity(eased);
        if progress <= 0.0 {
            styled = styled.invisible();
        }
        let mut element = styled.into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// A single animated boolean channel inside
/// [`AnimatedStateElement`]. Keeps an element-local clock so state
/// changes tween smoothly even though the underlying value is not
/// entity-backed.
#[derive(Clone)]
pub(crate) struct TweenChannel {
    start: Instant,
    previous_value: bool,
    value: f32,
}

impl TweenChannel {
    pub fn new(value: bool, now: Instant) -> Self {
        Self {
            start: now,
            previous_value: value,
            value: if value { 1.0 } else { 0.0 },
        }
    }

    /// Move `value` toward `target`. Returns `true` while still
    /// animating.
    pub fn update(&mut self, target: bool, config: &AnimationConfig, now: Instant) -> bool {
        if self.previous_value != target {
            self.start = now;
            self.previous_value = target;
        }
        let duration_secs = config.duration.as_secs_f32();
        let rate = if duration_secs > 0.0 {
            self.start.elapsed().as_secs_f32() / duration_secs
        } else {
            1.0
        };
        self.value = if target {
            (self.value + rate).min(1.0)
        } else {
            (self.value - rate).max(0.0)
        };
        (target && self.value < 1.0) || (!target && self.value > 0.0)
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Clone)]
struct AnimatedStateElementState {
    hover: TweenChannel,
    pressed: TweenChannel,
    checked: TweenChannel,
}

/// A custom element that wraps any styled element (a `Div` or a
/// `Stateful<Div>`) and drives three boolean channels — hovered,
/// pressed and checked — producing tweened progress values for a
/// renderer-supplied closure to interpolate against.
///
/// This is the WinUI renderer's replacement for CSS transitions:
/// instead of `.hover(|s| ...)` (instant) it lets a control animate
/// background/border/size/opacity between interaction states.
///
/// The element keeps its own element-local clock per channel, so
/// transitions are smooth even though the underlying state is stored
/// in a global map (hover/pressed) or a plain prop (checked).
pub struct AnimatedStateElement<E: Element + Styled + 'static> {
    /// Element identity for this wrapper's per-element state.
    pub id: ElementId,
    /// Id registered in [`WinUIInteractions`] (the interactive
    /// shell's ElementId) that drives hover/pressed.
    pub hover_id: ElementId,
    /// Whether the toggle/check channel is currently `true`. This is
    /// a construction-time snapshot; when the prop changes the element
    /// is rebuilt and the clock restarts, so the tween still works.
    pub checked: bool,
    pub config: AnimationConfig,
    child: Option<E>,
    render: Box<dyn Fn(E, f32, f32, f32) -> E>,
}

impl<E: Element + Styled + 'static> AnimatedStateElement<E> {
    pub fn new(
        id: impl Into<ElementId>,
        hover_id: impl Into<ElementId>,
        checked: bool,
        child: E,
        config: AnimationConfig,
        render: impl Fn(E, f32, f32, f32) -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            hover_id: hover_id.into(),
            checked,
            config,
            child: Some(child),
            render: Box::new(render),
        }
    }
}

impl<E: Element + Styled + 'static> IntoElement for AnimatedStateElement<E> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E: Element + Styled + 'static> Element for AnimatedStateElement<E> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let now = Instant::now();
        let hovered = interaction_hovered(cx, &self.hover_id);
        let pressed = interaction_pressed(cx, &self.hover_id);

        let (hover_progress, pressed_progress, checked_progress, is_animating) = window
            .with_element_state(
                global_id.unwrap(),
                |state: Option<AnimatedStateElementState>, _window| {
                    let mut state = state.unwrap_or(AnimatedStateElementState {
                        hover: TweenChannel::new(hovered, now),
                        pressed: TweenChannel::new(pressed, now),
                        checked: TweenChannel::new(self.checked, now),
                    });
                    let hover_animating = state.hover.update(hovered, &self.config, now);
                    let pressed_animating = state.pressed.update(pressed, &self.config, now);
                    let checked_animating = state.checked.update(self.checked, &self.config, now);
                    (
                        (
                            state.hover.value(),
                            state.pressed.value(),
                            state.checked.value(),
                            hover_animating || pressed_animating || checked_animating,
                        ),
                        state,
                    )
                },
            );

        if is_animating {
            window.request_animation_frame();
        }

        let eased_hover = (self.config.easing)(hover_progress);
        let eased_pressed = (self.config.easing)(pressed_progress);
        let eased_checked = (self.config.easing)(checked_progress);

        let child = self
            .child
            .take()
            .expect("AnimatedStateElement::request_layout called once");
        let mut element =
            (self.render)(child, eased_hover, eased_pressed, eased_checked).into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// A custom element that animates a child's left margin based on a
/// boolean value.
///
/// When `value` is `true` the child slides right by `distance`; when
/// `value` is `false` it slides back to `margin-left: 0`. This is used
/// for switch knobs.
pub struct AnimatedMarginElement {
    pub id: ElementId,
    pub value: bool,
    pub distance: Pixels,
    child: Option<Div>,
    pub config: AnimationConfig,
}

impl AnimatedMarginElement {
    pub fn new(id: impl Into<ElementId>, value: bool, distance: Pixels, child: Div) -> Self {
        Self {
            id: id.into(),
            value,
            distance,
            child: Some(child),
            config: AnimationConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }
}

impl IntoElement for AnimatedMarginElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for AnimatedMarginElement {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let now = Instant::now();
        let (progress, is_animating) = window.with_element_state(
            global_id.unwrap(),
            |state: Option<BooleanElementState>, _window| {
                let mut state = state.unwrap_or(BooleanElementState {
                    start: now,
                    previous_value: self.value,
                    previous_progress: if self.value { 1.0 } else { 0.0 },
                });
                if state.previous_value != self.value {
                    state.start = now;
                    state.previous_value = self.value;
                }
                let progress = state.progress(self.value, &self.config);
                let is_animating =
                    (self.value && progress < 1.0) || (!self.value && progress > 0.0);
                state.previous_progress = progress;
                ((progress, is_animating), state)
            },
        );

        if is_animating {
            window.request_animation_frame();
        }

        let eased = (self.config.easing)(progress);
        let distance_f: f32 = self.distance.into();
        let translate = distance_f * eased;

        let child = self
            .child
            .take()
            .expect("AnimatedMarginElement::request_layout called once");
        let mut element = child
            .id(self.id.clone())
            .ml(px(translate))
            .into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::hsla;

    #[test]
    fn interactions_track_hover_and_pressed_by_element_id() {
        let mut state = WinUIInteractions::default();
        let id: ElementId = "widget".into();
        assert!(!state.is_hovered(&id));
        state.set_hovered(id.clone(), true);
        assert!(state.is_hovered(&id));
        state.set_hovered(id.clone(), false);
        assert!(!state.is_hovered(&id));

        state.set_pressed(id.clone(), true);
        assert!(state.is_pressed(&id));
        assert!(!state.is_hovered(&id));
    }

    #[test]
    fn lerp_hsla_interpolates_rgba_endpoints() {
        let a = hsla(0., 0., 0., 0.);
        let b = hsla(0., 0., 1., 1.);
        let mid = lerp_hsla(a, b, 0.5);
        assert!((mid.l - 0.5).abs() < 0.02);
        assert!((mid.a - 0.5).abs() < 0.02);
        assert_eq!(lerp_hsla(a, b, 0.0), a);
        assert_eq!(lerp_hsla(a, b, 1.0), b);
    }

    #[test]
    fn tween_channel_advances_toward_target_and_back() {
        let config = AnimationConfig::default().with_duration(Duration::from_millis(100));
        let now = Instant::now();
        let mut ch = TweenChannel::new(false, now);
        assert!((ch.value() - 0.0).abs() < 0.001);
        ch.update(true, &config, now);
        assert!(ch.value() >= 0.0 && ch.value() <= 1.0);

        let far = now + Duration::from_secs(1);
        while ch.update(true, &config, far) {}
        assert!((ch.value() - 1.0).abs() < 0.001);
        while ch.update(false, &config, far + Duration::from_secs(1)) {}
        assert!((ch.value() - 0.0).abs() < 0.001);
    }

    #[test]
    fn lerp_f32_clamps_t() {
        assert_eq!(lerp_f32(0.0, 10.0, -1.0), 0.0);
        assert_eq!(lerp_f32(0.0, 10.0, 2.0), 10.0);
        assert_eq!(lerp_f32(0.0, 10.0, 0.5), 5.0);
    }
}
