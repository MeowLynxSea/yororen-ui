//! Presence / visibility animation data.
//!
//! This module is part of the headless data/control layer. It carries
//! *when* a component should be visible, *where* it is heading
//! (open/closed), and *how fast* it should get there. It does **not**
//! know anything about pixels, transforms, or renderers.
//!
//! Renderers in `yororen-ui-default-renderer` / `yororen-ui-winui-renderer`
//! read this state and apply the actual visual animation.

use std::time::Duration;

use super::config::AnimationConfig;
use super::timing::clamp01;

/// Data-only state that describes an open / closed transition.
///
/// `target` is the desired end state. `progress` is the current
/// normalized visibility [0.0, 1.0]. While `target != (progress == 1.0)`,
/// the renderer is responsible for advancing `progress` over time.
#[derive(Clone, Debug)]
pub struct AnimatedVisibility {
    /// Desired visibility. `true` means "should become / stay open".
    pub target: bool,
    /// Current normalized visibility progress.
    pub progress: f32,
    /// Configuration used while entering (target `true`).
    pub enter_config: AnimationConfig,
    /// Configuration used while exiting (target `false`).
    pub exit_config: AnimationConfig,
}

impl Default for AnimatedVisibility {
    fn default() -> Self {
        Self {
            target: false,
            progress: 0.0,
            enter_config: AnimationConfig::default(),
            exit_config: AnimationConfig::default(),
        }
    }
}

impl AnimatedVisibility {
    /// Create a new closed visibility with default enter/exit configs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new open visibility, already at progress 1.0.
    pub fn open() -> Self {
        Self {
            target: true,
            progress: 1.0,
            ..Default::default()
        }
    }

    /// Use `enter` for both directions.
    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.enter_config = config.clone();
        self.exit_config = config;
        self
    }

    /// Set the enter config.
    pub fn with_enter(mut self, config: AnimationConfig) -> Self {
        self.enter_config = config;
        self
    }

    /// Set the exit config.
    pub fn with_exit(mut self, config: AnimationConfig) -> Self {
        self.exit_config = config;
        self
    }

    /// Target the open state.
    pub fn show(&mut self) {
        self.target = true;
    }

    /// Target the closed state.
    pub fn hide(&mut self) {
        self.target = false;
    }

    /// Toggle the target state.
    pub fn toggle(&mut self) {
        self.target = !self.target;
    }

    /// The desired end state.
    pub fn is_open(&self) -> bool {
        self.target
    }

    /// Whether this component should still be mounted in the tree.
    ///
    /// A closing component stays visible until its exit animation
    /// finishes (`progress > 0.0`).
    pub fn is_visible(&self) -> bool {
        self.target || self.progress > 0.0
    }

    /// Whether the visibility is currently changing.
    pub fn is_animating(&self) -> bool {
        (self.target && self.progress < 1.0) || (!self.target && self.progress > 0.0)
    }

    /// Current phase of the visibility lifecycle.
    pub fn phase(&self) -> AnimationPhase {
        if self.target {
            if self.progress >= 1.0 {
                AnimationPhase::Open
            } else {
                AnimationPhase::Enter
            }
        } else if self.progress <= 0.0 {
            AnimationPhase::Closed
        } else {
            AnimationPhase::Exit
        }
    }

    /// Advance progress by `dt` toward the current target.
    ///
    /// Returns `true` if the animation is still running after the
    /// update (i.e. the caller should request another frame).
    pub fn update(&mut self, dt: Duration) -> bool {
        let config = if self.target {
            &self.enter_config
        } else {
            &self.exit_config
        };

        let duration_secs = config.duration.as_secs_f32();
        let rate = if duration_secs > 0.0 {
            dt.as_secs_f32() / duration_secs
        } else {
            1.0
        };

        if self.target {
            self.progress = (self.progress + rate).min(1.0);
        } else {
            self.progress = (self.progress - rate).max(0.0);
        }

        self.is_animating()
    }

    /// Jump directly to a specific progress.
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = clamp01(progress);
    }
}

/// Lifecycle phase of an [`AnimatedVisibility`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationPhase {
    /// Transitioning from closed to open.
    Enter,
    /// Fully open and idle.
    Open,
    /// Transitioning from open to closed.
    Exit,
    /// Fully closed and idle.
    Closed,
}

/// Trait for state entities that own an [`AnimatedVisibility`].
///
/// Implemented by the headless state structs (`ModalState`,
/// `DropdownMenuState`, `PopoverState`, etc.) so renderer-side
/// animation drivers can be generic over the concrete state type.
///
/// The bound is only `'static` because `gpui::Entity<S>` already
/// owns the synchronization semantics; the trait itself just reads
/// and writes the visibility field.
pub trait AnimatedPresenceState: 'static {
    /// Borrow the visibility state.
    fn visibility(&self) -> &AnimatedVisibility;
    /// Mutably borrow the visibility state.
    fn visibility_mut(&mut self) -> &mut AnimatedVisibility;
}
