//! Animation helper functions and macros.

use std::time::Duration;

use gpui::{Div, ElementId, Hsla, InteractiveElement, Pixels, Stateful, Styled};

/// Extension trait for animating gpui elements.
pub trait AnimateExt {
    /// Apply a fade animation.
    fn animate_fade(
        self,
        id: impl Into<ElementId>,
        duration: Duration,
        progress: f32,
    ) -> Stateful<Self>
    where
        Self: Sized;

    /// Apply a slide animation from direction.
    fn animate_slide(
        self,
        id: impl Into<ElementId>,
        direction: SlideDirection,
        distance: Pixels,
        duration: Duration,
        progress: f32,
    ) -> Stateful<Self>
    where
        Self: Sized;

    /// Apply a scale animation.
    fn animate_scale(
        self,
        id: impl Into<ElementId>,
        duration: Duration,
        progress: f32,
    ) -> Stateful<Self>
    where
        Self: Sized;
}

/// Direction for slide animations.
#[derive(Debug, Clone, Copy)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

impl AnimateExt for Div {
    fn animate_fade(
        self,
        id: impl Into<ElementId>,
        _duration: Duration,
        progress: f32,
    ) -> Stateful<Self> {
        self.id(id).opacity(progress)
    }

    fn animate_slide(
        self,
        id: impl Into<ElementId>,
        direction: SlideDirection,
        distance: Pixels,
        _duration: Duration,
        progress: f32,
    ) -> Stateful<Self> {
        let distance_f: f32 = distance.into();
        let (ml, mt) = match direction {
            SlideDirection::Left => (gpui::px(distance_f * (progress - 1.0)), gpui::px(0.0)),
            SlideDirection::Right => (gpui::px(distance_f * (1.0 - progress)), gpui::px(0.0)),
            SlideDirection::Up => (gpui::px(0.0), gpui::px(distance_f * (progress - 1.0))),
            SlideDirection::Down => (gpui::px(0.0), gpui::px(distance_f * (1.0 - progress))),
        };
        self.id(id).opacity(progress).ml(ml).mt(mt)
    }

    fn animate_scale(
        self,
        id: impl Into<ElementId>,
        _duration: Duration,
        progress: f32,
    ) -> Stateful<Self> {
        self.id(id).opacity(progress)
    }
}

/// Helper to interpolate between two values.
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

/// Helper to interpolate between two colors.
pub fn lerp_color(start: Hsla, end: Hsla, t: f32) -> Hsla {
    Hsla {
        h: lerp(start.h, end.h, t),
        s: lerp(start.s, end.s, t),
        l: lerp(start.l, end.l, t),
        a: lerp(start.a, end.a, t),
    }
}

/// Create a simple animation id from a prefix and state.
pub fn animation_id(prefix: &str, state: impl std::fmt::Debug) -> String {
    format!("{}:{:?}", prefix, state)
}
