//! Animation orchestration.
//!
//! This module provides building blocks for sequencing and parallelizing
//! animations while staying compatible with gpui's `AnimationExt::with_animations`.
//!
//! The key idea is to compile orchestration into a `Vec<gpui::Animation>` for the
//! outer timeline and provide helpers to map the current `(animation_index, delta)`
//! into per-track progress.

use std::time::Duration;

use gpui::Animation;

use super::timing::{clamp01, parallel_progress, sequence_progress};

/// A named handle for a track inside an orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackId(pub usize);

#[derive(Debug, Clone, Copy)]
struct TrackSpec {
    duration: Duration,
    delay: Duration,
}

/// High-level orchestration builder.
///
/// Build up a timeline consisting of *steps*; each step can contain one
/// sequential track or multiple parallel tracks.
#[derive(Debug, Default, Clone)]
pub struct Orchestration {
    steps: Vec<Step>,
}

#[derive(Debug, Default, Clone)]
struct Step {
    tracks: Vec<TrackSpec>,
}

impl Orchestration {
    /// Create a new empty orchestration.
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Add a sequential step.
    ///
    /// The returned `TrackId` can be used to query progress in the animator.
    pub fn then(self, duration: Duration) -> (Self, TrackId) {
        self.then_delayed(duration, Duration::ZERO)
    }

    /// Add a sequential step with a per-track delay.
    ///
    /// The step's total duration is `delay + duration`.
    pub fn then_delayed(mut self, duration: Duration, delay: Duration) -> (Self, TrackId) {
        let track_id = TrackId(self.total_tracks());
        self.steps.push(Step {
            tracks: vec![TrackSpec { duration, delay }],
        });
        (self, track_id)
    }

    /// Add a parallel step containing multiple tracks.
    ///
    /// Each track progresses relative to the longest track within the step.
    pub fn with_parallel(
        self,
        durations: impl IntoIterator<Item = Duration>,
    ) -> (Self, Vec<TrackId>) {
        let items = durations
            .into_iter()
            .map(|duration| (duration, Duration::ZERO));
        self.with_parallel_delayed(items)
    }

    /// Add a parallel step containing multiple tracks, each with its own delay.
    ///
    /// Step duration is based on the maximum of `delay + duration` across tracks.
    pub fn with_parallel_delayed(
        mut self,
        tracks: impl IntoIterator<Item = (Duration, Duration)>,
    ) -> (Self, Vec<TrackId>) {
        let mut ids = Vec::new();
        let mut specs = Vec::new();
        for (duration, delay) in tracks {
            let track_id = TrackId(self.total_tracks() + specs.len());
            ids.push(track_id);
            specs.push(TrackSpec { duration, delay });
        }
        self.steps.push(Step { tracks: specs });
        (self, ids)
    }

    /// Add a delay-only step.
    pub fn delay(mut self, duration: Duration) -> Self {
        if duration.is_zero() {
            return self;
        }

        self.steps.push(Step {
            tracks: vec![TrackSpec {
                duration,
                delay: Duration::ZERO,
            }],
        });
        self
    }

    /// Total number of steps.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Total number of tracks.
    pub fn track_count(&self) -> usize {
        self.total_tracks()
    }

    fn total_tracks(&self) -> usize {
        self.steps.iter().map(|s| s.tracks.len()).sum()
    }

    /// Compile into gpui animations for use with `with_animations`.
    ///
    /// Each orchestration step maps to one gpui `Animation`.
    pub fn compile(&self) -> Vec<Animation> {
        self.steps
            .iter()
            .map(|step| Animation::new(step_duration(step)))
            .collect()
    }

    /// Compute progress for a specific track given the current step index and delta.
    ///
    /// - `animation_index` and `delta` are the values passed by gpui's `with_animations` animator.
    /// - `track_id` refers to the track returned by [`then`] / [`with_parallel`].
    ///
    /// This method is **global** over the whole orchestration timeline:
    /// you may call it for any track on any step, and it will return:
    ///
    /// - `0.0` before the track starts
    /// - `1.0` after it finishes
    /// - a normalized value while active
    pub fn track_progress(&self, animation_index: usize, delta: f32, track_id: TrackId) -> f32 {
        let delta = clamp01(delta);

        let Some(step) = self.steps.get(animation_index) else {
            return 1.0;
        };

        // Convert gpui's step-local delta into a global elapsed time.
        let mut elapsed = Duration::ZERO;
        for step in self.steps.iter().take(animation_index) {
            elapsed += step_duration(step);
        }

        let current_step_duration = step_duration(step);
        if !current_step_duration.is_zero() {
            elapsed += Duration::from_secs_f32(delta * current_step_duration.as_secs_f32());
        }

        // Locate the requested track globally.
        let Some((track_step_index, track)) = self.locate_track(track_id) else {
            return 1.0;
        };

        // Track start is the start of its step + per-track delay.
        let mut track_step_start = Duration::ZERO;
        for step in self.steps.iter().take(track_step_index) {
            track_step_start += step_duration(step);
        }
        let track_start = track_step_start + track.delay;

        track_progress_from_elapsed(elapsed, track_start, track.duration)
    }

    fn locate_track(&self, track_id: TrackId) -> Option<(usize, &TrackSpec)> {
        let mut current = 0usize;

        for (step_index, step) in self.steps.iter().enumerate() {
            for track in &step.tracks {
                if current == track_id.0 {
                    return Some((step_index, track));
                }
                current += 1;
            }
        }

        None
    }
}

fn step_duration(step: &Step) -> Duration {
    step.tracks
        .iter()
        .map(|t| t.delay + t.duration)
        .max()
        .unwrap_or(Duration::ZERO)
}

fn track_progress_from_elapsed(elapsed: Duration, start: Duration, duration: Duration) -> f32 {
    if elapsed < start {
        return 0.0;
    }

    if duration.is_zero() {
        return 1.0;
    }

    let active_elapsed = elapsed - start;
    clamp01(active_elapsed.as_secs_f32() / duration.as_secs_f32())
}

// ============================================================================
// Backwards-compatible API
// ============================================================================

/// A builder for sequencing animations by duration.
///
/// This is retained for compatibility with earlier docs, but prefer [`Orchestration`]
/// when integrating with gpui.
#[derive(Debug, Default, Clone)]
pub struct AnimationSequence {
    durations: Vec<Duration>,
}

impl AnimationSequence {
    pub fn new() -> Self {
        Self {
            durations: Vec::new(),
        }
    }

    pub fn then(mut self, duration: Duration) -> Self {
        self.durations.push(duration);
        self
    }

    pub fn then_all(mut self, durations: impl IntoIterator<Item = Duration>) -> Self {
        self.durations.extend(durations);
        self
    }

    pub fn total_duration(&self) -> Duration {
        self.durations.iter().copied().sum()
    }

    pub fn len(&self) -> usize {
        self.durations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.durations.is_empty()
    }

    pub fn calculate_progress(&self, total_progress: f32) -> (usize, f32) {
        sequence_progress(&self.durations, total_progress)
    }
}

/// A builder for parallel animations by duration.
///
/// This is retained for compatibility with earlier docs, but prefer [`Orchestration`]
/// when integrating with gpui.
#[derive(Debug, Default, Clone)]
pub struct AnimationParallel {
    durations: Vec<Duration>,
}

impl AnimationParallel {
    pub fn new() -> Self {
        Self {
            durations: Vec::new(),
        }
    }

    pub fn with(mut self, duration: Duration) -> Self {
        self.durations.push(duration);
        self
    }

    pub fn with_all(mut self, durations: impl IntoIterator<Item = Duration>) -> Self {
        self.durations.extend(durations);
        self
    }

    pub fn max_duration(&self) -> Duration {
        self.durations
            .iter()
            .copied()
            .max()
            .unwrap_or(Duration::ZERO)
    }

    pub fn len(&self) -> usize {
        self.durations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.durations.is_empty()
    }

    pub fn calculate_progress(&self, total_progress: f32, animation_index: usize) -> f32 {
        parallel_progress(&self.durations, total_progress, animation_index)
    }
}

// Convenience functions

pub fn sequence(durations: &[Duration]) -> AnimationSequence {
    AnimationSequence::new().then_all(durations.iter().copied())
}

pub fn parallel(durations: &[Duration]) -> AnimationParallel {
    AnimationParallel::new().with_all(durations.iter().copied())
}

/// A trait for creating staggered durations.
pub trait Staggered {
    fn stagger(self, item_count: usize, delay: Duration) -> Vec<Duration>;
}

impl Staggered for Duration {
    fn stagger(self, item_count: usize, delay: Duration) -> Vec<Duration> {
        let base_ms = self.as_millis() as f32;
        let delay_ms = delay.as_millis() as f32;

        (0..item_count)
            .map(|i| Duration::from_millis((base_ms + i as f32 * delay_ms) as u64))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx(actual: f32, expected: f32) {
        let diff = (actual - expected).abs();
        assert!(
            diff <= 1e-4,
            "expected {expected} but got {actual} (diff={diff})"
        );
    }

    #[test]
    fn sequential_track_delay_is_applied() {
        let (orch, track) = Orchestration::new()
            .then_delayed(Duration::from_millis(100), Duration::from_millis(50));

        // Step duration is 150ms.
        assert_eq!(orch.compile().len(), 1);

        // Before delay.
        assert_approx(orch.track_progress(0, 0.0, track), 0.0);
        assert_approx(orch.track_progress(0, 50.0 / 150.0, track), 0.0);

        // 25ms into active animation => 0.25.
        assert_approx(orch.track_progress(0, 75.0 / 150.0, track), 0.25);

        // End.
        assert_approx(orch.track_progress(0, 1.0, track), 1.0);
    }

    #[test]
    fn parallel_tracks_can_have_individual_delays() {
        let (orch, tracks) = Orchestration::new().with_parallel_delayed([
            (Duration::from_millis(100), Duration::from_millis(0)),
            (Duration::from_millis(100), Duration::from_millis(50)),
        ]);
        let a = tracks[0];
        let b = tracks[1];

        // Step duration is max(100, 150)=150ms.
        assert_eq!(orch.compile().len(), 1);

        // At 50ms elapsed: a is half done, b hasn't started.
        let delta = 50.0 / 150.0;
        assert_approx(orch.track_progress(0, delta, a), 0.5);
        assert_approx(orch.track_progress(0, delta, b), 0.0);

        // At 100ms elapsed: a is done, b is half done.
        let delta = 100.0 / 150.0;
        assert_approx(orch.track_progress(0, delta, a), 1.0);
        assert_approx(orch.track_progress(0, delta, b), 0.5);
    }

    #[test]
    fn track_progress_is_global_across_steps() {
        let (orch, a) = Orchestration::new().then(Duration::from_millis(100));
        let (orch, b) = orch.then(Duration::from_millis(100));

        assert_approx(orch.track_progress(0, 0.5, a), 0.5);
        assert_approx(orch.track_progress(0, 0.5, b), 0.0);

        assert_approx(orch.track_progress(1, 0.5, a), 1.0);
        assert_approx(orch.track_progress(1, 0.5, b), 0.5);
    }
}
