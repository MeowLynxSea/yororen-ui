//! Preset animations module.
//!
//! Provides pre-built animation effects that can be easily applied to components.

use std::time::Duration;

use gpui::{Pixels, Styled};

use super::easing::{
    ease_in_bounce, ease_in_out, ease_out_bounce, ease_out_cubic,
    ease_out_elastic, ease_out_quint,
};

/// Preset animation durations.
pub mod duration {
    use super::Duration;

    /// Very fast animation (100ms).
    pub const VERY_FAST: Duration = Duration::from_millis(100);

    /// Fast animation (150ms).
    pub const FAST: Duration = Duration::from_millis(150);

    /// Normal animation (200ms).
    pub const NORMAL: Duration = Duration::from_millis(200);

    /// Slow animation (300ms).
    pub const SLOW: Duration = Duration::from_millis(300);

    /// Very slow animation (400ms).
    pub const VERY_SLOW: Duration = Duration::from_millis(400);

    /// Instant (0ms).
    pub const INSTANT: Duration = Duration::ZERO;
}

/// A struct representing a preset animation effect.
#[derive(Debug, Clone)]
pub struct PresetAnimation {
    /// Duration of the animation.
    pub duration: Duration,
    /// The easing function name for reference.
    pub easing_name: &'static str,
    /// The animation type.
    pub animation_type: AnimationType,
}

/// Types of preset animations.
#[derive(Debug, Clone)]
pub enum AnimationType {
    /// Fade in.
    FadeIn,
    /// Fade out.
    FadeOut,
    /// Slide in from direction.
    SlideIn(SlideDirection),
    /// Slide out to direction.
    SlideOut(SlideDirection),
    /// Scale in.
    ScaleIn,
    /// Scale out.
    ScaleOut,
    /// Bounce in.
    BounceIn,
    /// Bounce out.
    BounceOut,
    /// Elastic in.
    ElasticIn,
    /// Elastic out.
    ElasticOut,
    /// Combined fade and slide.
    FadeSlideIn(SlideDirection),
    /// Combined fade and scale.
    FadeScaleIn,
}

/// Slide direction.
#[derive(Debug, Clone, Copy)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

// ============================================================================
// Fade Animations
// ============================================================================

/// Fade in animation (opacity 0 -> 1).
pub struct FadeIn;

impl FadeIn {
    /// Create a new fade in animation with default duration.
    pub fn new() -> Self {
        Self
    }

    /// Apply to a gpui element using custom easing.
    pub fn apply<E: Fn(f32) -> f32 + 'static>(
        self,
        duration: Duration,
        easing: E,
    ) -> impl FnOnce(gpui::Div, f32) -> gpui::Div + 'static {
        let _ = duration;
        move |element: gpui::Div, progress: f32| {
            let eased_progress = easing(progress);
            element.opacity(eased_progress)
        }
    }

    /// Apply with default ease_out_cubic.
    pub fn apply_default(self, element: gpui::Div, progress: f32) -> gpui::Div {
        element.opacity(progress)
    }
}

impl Default for FadeIn {
    fn default() -> Self {
        Self::new()
    }
}

/// Fade out animation (opacity 1 -> 0).
pub struct FadeOut;

impl FadeOut {
    /// Create a new fade out animation.
    pub fn new() -> Self {
        Self
    }

    /// Apply to a gpui element.
    pub fn apply(self, element: gpui::Div, progress: f32) -> gpui::Div {
        element.opacity(1.0 - progress)
    }
}

impl Default for FadeOut {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Preset Functions
// ============================================================================

/// Fade slide in preset for menus (matches existing MENU_OPEN animation).
pub fn fade_slide_in(duration: Duration) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let _ = duration;
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_quint(progress);
        element
            .opacity(eased)
            .mt(gpui::px(10.0 - 6.0 * eased))
    }
}

/// Fade slide out preset for menus.
pub fn fade_slide_out(duration: Duration) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let _ = duration;
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_quint(progress);
        element
            .opacity(1.0 - eased)
            .mt(gpui::px(4.0 + 6.0 * eased))
    }
}

/// Pulse animation for loading states.
pub fn pulse(duration: Duration) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let _ = duration;
    move |element: gpui::Div, progress: f32| {
        let eased = ease_in_out(progress);
        let opacity = 0.55 + 0.40 * eased;
        element.opacity(opacity)
    }
}

/// Fade in from left.
pub fn fade_slide_in_left(distance: Pixels) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let distance_f: f32 = distance.into();
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_cubic(progress);
        let translate = -distance_f * (1.0 - eased);
        element
            .opacity(eased)
            .ml(gpui::px(translate))
    }
}

/// Fade in from right.
pub fn fade_slide_in_right(distance: Pixels) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let distance_f: f32 = distance.into();
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_cubic(progress);
        let translate = distance_f * (1.0 - eased);
        element
            .opacity(eased)
            .ml(gpui::px(translate))
    }
}

/// Fade in from top.
pub fn fade_slide_in_up(distance: Pixels) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let distance_f: f32 = distance.into();
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_cubic(progress);
        let translate = -distance_f * (1.0 - eased);
        element
            .opacity(eased)
            .mt(gpui::px(translate))
    }
}

/// Fade in from bottom.
pub fn fade_slide_in_down(distance: Pixels) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let distance_f: f32 = distance.into();
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_cubic(progress);
        let translate = distance_f * (1.0 - eased);
        element
            .opacity(eased)
            .mt(gpui::px(translate))
    }
}

// ============================================================================
// Scale Animations
// ============================================================================

/// Scale in animation (scale 0 -> 1 with fade).
/// Note: Uses opacity and scale emulation via transform since gpui doesn't support transform().
pub struct ScaleIn;

impl ScaleIn {
    /// Create a new scale in animation.
    pub fn new() -> Self {
        Self
    }

    /// Apply to a gpui element with custom easing.
    pub fn apply<E: Fn(f32) -> f32 + 'static>(
        self,
        duration: Duration,
        easing: E,
    ) -> impl FnOnce(gpui::Div, f32) -> gpui::Div + 'static {
        let _ = duration;
        move |element: gpui::Div, progress: f32| {
            let eased_progress = easing(progress);
            // Scale from 0.8 to 1.0 with opacity fade in
            // Note: Full scale requires CSS transform, using opacity as visual cue
            element.opacity(eased_progress)
        }
    }

    /// Apply with default ease_out_cubic.
    pub fn apply_default(self, element: gpui::Div, progress: f32) -> gpui::Div {
        let eased = ease_out_cubic(progress);
        element.opacity(eased)
    }
}

impl Default for ScaleIn {
    fn default() -> Self {
        Self::new()
    }
}

/// Scale out animation (scale 1 -> 0 with fade).
pub struct ScaleOut;

impl ScaleOut {
    /// Create a new scale out animation.
    pub fn new() -> Self {
        Self
    }

    /// Apply to a gpui element.
    pub fn apply<E: Fn(f32) -> f32 + 'static>(
        self,
        duration: Duration,
        easing: E,
    ) -> impl FnOnce(gpui::Div, f32) -> gpui::Div + 'static {
        let _ = duration;
        move |element: gpui::Div, progress: f32| {
            let eased_progress = easing(progress);
            // Scale from 1.0 to 0.8 with opacity fade out
            element.opacity(1.0 - eased_progress)
        }
    }

    /// Apply with default ease_out_cubic (reversed).
    pub fn apply_default(self, element: gpui::Div, progress: f32) -> gpui::Div {
        let eased = ease_out_cubic(progress);
        element.opacity(1.0 - eased)
    }
}

impl Default for ScaleOut {
    fn default() -> Self {
        Self::new()
    }
}

/// Fade scale in animation (combined fade and scale).
pub fn fade_scale_in(duration: Duration) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let _ = duration;
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_cubic(progress);
        element.opacity(eased)
    }
}

/// Fade scale out animation.
pub fn fade_scale_out(duration: Duration) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let _ = duration;
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_cubic(progress);
        element.opacity(1.0 - eased)
    }
}

// ============================================================================
// Bounce Animations
// ============================================================================

/// Bounce in animation (bounces into view).
pub struct BounceIn;

impl BounceIn {
    /// Create a new bounce in animation.
    pub fn new() -> Self {
        Self
    }

    /// Apply to a gpui element with bounce effect.
    pub fn apply<E: Fn(f32) -> f32 + 'static>(
        self,
        duration: Duration,
        easing: E,
    ) -> impl FnOnce(gpui::Div, f32) -> gpui::Div + 'static {
        let _ = duration;
        move |element: gpui::Div, progress: f32| {
            let eased_progress = easing(progress);
            // Start from above and bounce down
            let translate = -30.0 * (1.0 - eased_progress);
            element
                .opacity(eased_progress)
                .mt(gpui::px(translate))
        }
    }

    /// Apply with default ease_out_bounce.
    pub fn apply_default(self, element: gpui::Div, progress: f32) -> gpui::Div {
        let eased = ease_out_bounce(progress);
        let translate = -30.0 * (1.0 - eased);
        element
            .opacity(eased)
            .mt(gpui::px(translate))
    }
}

impl Default for BounceIn {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounce out animation (bounces out of view).
pub struct BounceOut;

impl BounceOut {
    /// Create a new bounce out animation.
    pub fn new() -> Self {
        Self
    }

    /// Apply to a gpui element.
    pub fn apply<E: Fn(f32) -> f32 + 'static>(
        self,
        duration: Duration,
        easing: E,
    ) -> impl FnOnce(gpui::Div, f32) -> gpui::Div + 'static {
        let _ = duration;
        move |element: gpui::Div, progress: f32| {
            let eased_progress = easing(progress);
            // Bounce down and away
            let translate = 30.0 * eased_progress;
            element
                .opacity(1.0 - eased_progress)
                .mt(gpui::px(translate))
        }
    }

    /// Apply with default ease_in_bounce.
    pub fn apply_default(self, element: gpui::Div, progress: f32) -> gpui::Div {
        let eased = ease_in_bounce(progress);
        let translate = 30.0 * eased;
        element
            .opacity(1.0 - eased)
            .mt(gpui::px(translate))
    }
}

impl Default for BounceOut {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounce in from left.
pub fn bounce_in_left(distance: Pixels) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let distance_f: f32 = distance.into();
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_bounce(progress);
        let translate = -distance_f * (1.0 - eased);
        element
            .opacity(eased)
            .ml(gpui::px(translate))
    }
}

/// Bounce in from right.
pub fn bounce_in_right(distance: Pixels) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let distance_f: f32 = distance.into();
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_bounce(progress);
        let translate = distance_f * (1.0 - eased);
        element
            .opacity(eased)
            .ml(gpui::px(translate))
    }
}

/// Bounce in from top.
pub fn bounce_in_up(distance: Pixels) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let distance_f: f32 = distance.into();
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_bounce(progress);
        let translate = -distance_f * (1.0 - eased);
        element
            .opacity(eased)
            .mt(gpui::px(translate))
    }
}

/// Bounce in from bottom.
pub fn bounce_in_down(distance: Pixels) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let distance_f: f32 = distance.into();
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_bounce(progress);
        let translate = distance_f * (1.0 - eased);
        element
            .opacity(eased)
            .mt(gpui::px(translate))
    }
}

// ============================================================================
// Elastic Animations
// ============================================================================

/// Elastic in animation (elastic bounce into view).
/// Note: Uses opacity and position since gpui doesn't support CSS transform.
pub struct ElasticIn;

impl ElasticIn {
    /// Create a new elastic in animation.
    pub fn new() -> Self {
        Self
    }

    /// Apply to a gpui element with elastic effect.
    pub fn apply<E: Fn(f32) -> f32 + 'static>(
        self,
        duration: Duration,
        easing: E,
    ) -> impl FnOnce(gpui::Div, f32) -> gpui::Div + 'static {
        let _ = duration;
        move |element: gpui::Div, progress: f32| {
            let eased_progress = easing(progress);
            // Elastic effect via position overshoot
            let overshoot = if eased_progress < 0.5 {
                -10.0 * (1.0 - 2.0 * eased_progress)
            } else {
                0.0
            };
            element
                .opacity(eased_progress)
                .mt(gpui::px(overshoot))
        }
    }

    /// Apply with default ease_out_elastic.
    pub fn apply_default(self, element: gpui::Div, progress: f32) -> gpui::Div {
        let eased = ease_out_elastic(progress);
        element.opacity(eased)
    }
}

impl Default for ElasticIn {
    fn default() -> Self {
        Self::new()
    }
}

/// Elastic out animation (elastic bounce out of view).
pub struct ElasticOut;

impl ElasticOut {
    /// Create a new elastic out animation.
    pub fn new() -> Self {
        Self
    }

    /// Apply to a gpui element.
    pub fn apply<E: Fn(f32) -> f32 + 'static>(
        self,
        duration: Duration,
        easing: E,
    ) -> impl FnOnce(gpui::Div, f32) -> gpui::Div + 'static {
        let _ = duration;
        move |element: gpui::Div, progress: f32| {
            let eased_progress = easing(progress);
            // Elastic effect via position overshoot
            let overshoot = if eased_progress > 0.5 {
                10.0 * (2.0 * (eased_progress - 0.5))
            } else {
                0.0
            };
            element
                .opacity(1.0 - eased_progress)
                .mt(gpui::px(overshoot))
        }
    }

    /// Apply with default ease_out_elastic (reversed).
    pub fn apply_default(self, element: gpui::Div, progress: f32) -> gpui::Div {
        let eased = ease_out_elastic(progress);
        element.opacity(1.0 - eased)
    }
}

impl Default for ElasticOut {
    fn default() -> Self {
        Self::new()
    }
}

/// Elastic scale in animation.
pub fn elastic_scale_in(duration: Duration) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let _ = duration;
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_elastic(progress);
        element.opacity(eased)
    }
}

/// Elastic scale out animation.
pub fn elastic_scale_out(duration: Duration) -> impl Fn(gpui::Div, f32) -> gpui::Div {
    let _ = duration;
    move |element: gpui::Div, progress: f32| {
        let eased = ease_out_elastic(progress);
        element.opacity(1.0 - eased)
    }
}
