//! Animation configuration module.

use std::time::Duration;

use super::easing::EasingFn;
use super::easing::ease_out_quad;

/// Configuration for animations.
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Duration of the animation.
    pub duration: Duration,
    /// Easing function to use.
    pub easing: EasingFn,
    /// Delay before starting the animation.
    pub delay: Duration,
    /// Whether the animation should repeat.
    pub repeat: bool,
    /// Whether the animation should reverse (yoyo effect).
    pub reverse: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(200),
            easing: ease_out_quad,
            delay: Duration::ZERO,
            repeat: false,
            reverse: false,
        }
    }
}

impl AnimationConfig {
    /// Create a new animation config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the easing function.
    pub fn with_easing(mut self, easing: EasingFn) -> Self {
        self.easing = easing;
        self
    }

    /// Set the delay.
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Enable repeat.
    pub fn with_repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    /// Enable reverse (yoyo).
    pub fn with_reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    /// Convert to gpui Animation.
    pub fn to_gpui_animation(self) -> gpui::Animation {
        let mut animation = gpui::Animation::new(self.duration);

        if self.repeat {
            animation = animation.repeat();
        }

        // Note: gpui's Animation doesn't expose custom easing directly,
        // it uses the built-in easing functions. We'll use a wrapper approach
        // for custom easing in the preset module.
        animation
    }
}

/// State tracking for complex animations.
#[derive(Debug, Clone, Default)]
pub struct AnimationState {
    /// Current progress (0.0 to 1.0).
    pub progress: f32,
    /// Whether the animation is currently running.
    pub is_running: bool,
    /// Whether the animation is paused.
    pub is_paused: bool,
}

impl AnimationState {
    /// Create a new animation state.
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            is_running: false,
            is_paused: false,
        }
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.is_running = false;
        self.is_paused = false;
    }

    /// Update progress value.
    pub fn update(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }
}
