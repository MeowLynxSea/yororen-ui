//! Animation orchestration module.
//!
//! Provides capabilities for sequencing and parallelizing multiple animations.

use std::time::Duration;

/// A builder for sequencing animations.
pub struct AnimationSequence {
    animations: Vec<SequenceItem>,
    total_duration: Duration,
}

struct SequenceItem {
    duration: Duration,
    // The actual animation logic is applied when with_animation is called
    // This is a simplified representation
}

impl AnimationSequence {
    /// Create a new animation sequence.
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            total_duration: Duration::ZERO,
        }
    }

    /// Add an animation to the sequence.
    pub fn then(mut self, duration: Duration) -> Self {
        self.total_duration += duration;
        self.animations.push(SequenceItem { duration });
        self
    }

    /// Add multiple animations at once.
    pub fn then_all(mut self, durations: impl IntoIterator<Item = Duration>) -> Self {
        for duration in durations {
            self = self.then(duration);
        }
        self
    }

    /// Get the total duration of the sequence.
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    /// Get the number of animations in the sequence.
    pub fn len(&self) -> usize {
        self.animations.len()
    }

    /// Check if the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }

    /// Calculate the progress for a specific animation in the sequence.
    /// Returns (animation_index, animation_progress) or (len, 1.0) if complete.
    pub fn calculate_progress(&self, total_progress: f32) -> (usize, f32) {
        if self.is_empty() {
            return (0, 1.0);
        }

        let total_ms = self.total_duration.as_millis() as f32;
        if total_ms == 0.0 {
            return (self.animations.len() - 1, 1.0);
        }

        let current_ms = total_progress * total_ms;
        let mut accumulated_ms = 0.0;

        for (i, item) in self.animations.iter().enumerate() {
            let item_duration_ms = item.duration.as_millis() as f32;
            if current_ms < accumulated_ms + item_duration_ms {
                let item_progress = (current_ms - accumulated_ms) / item_duration_ms;
                return (i, item_progress.clamp(0.0, 1.0));
            }
            accumulated_ms += item_duration_ms;
        }

        (self.animations.len(), 1.0)
    }
}

impl Default for AnimationSequence {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for parallel animations.
pub struct AnimationParallel {
    animations: Vec<ParallelItem>,
    max_duration: Duration,
}

struct ParallelItem {
    duration: Duration,
}

impl AnimationParallel {
    /// Create a new parallel animation builder.
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            max_duration: Duration::ZERO,
        }
    }

    /// Add an animation to run in parallel.
    pub fn with(mut self, duration: Duration) -> Self {
        if duration > self.max_duration {
            self.max_duration = duration;
        }
        self.animations.push(ParallelItem { duration });
        self
    }

    /// Add multiple animations to run in parallel.
    pub fn with_all(mut self, durations: impl IntoIterator<Item = Duration>) -> Self {
        for duration in durations {
            self = self.with(duration);
        }
        self
    }

    /// Get the maximum duration of all parallel animations.
    pub fn max_duration(&self) -> Duration {
        self.max_duration
    }

    /// Get the number of parallel animations.
    pub fn len(&self) -> usize {
        self.animations.len()
    }

    /// Check if there are no parallel animations.
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }

    /// Calculate the progress for a specific animation in parallel.
    /// All animations share the same total progress.
    pub fn calculate_progress(&self, total_progress: f32, animation_index: usize) -> f32 {
        if self.is_empty() || animation_index >= self.animations.len() {
            return 1.0;
        }

        let item = &self.animations[animation_index];
        if self.max_duration.is_zero() {
            return 1.0;
        }

        // Scale progress based on relative duration
        let ratio = item.duration.as_millis() as f32 / self.max_duration.as_millis() as f32;
        (total_progress / ratio).clamp(0.0, 1.0)
    }
}

impl Default for AnimationParallel {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Create a sequence with the given durations.
pub fn sequence(durations: &[Duration]) -> AnimationSequence {
    let mut seq = AnimationSequence::new();
    for &duration in durations {
        seq = seq.then(duration);
    }
    seq
}

/// Create parallel animations with the given durations.
pub fn parallel(durations: &[Duration]) -> AnimationParallel {
    let mut par = AnimationParallel::new();
    for &duration in durations {
        par = par.with(duration);
    }
    par
}

/// A trait for creating staggered animations.
pub trait Staggered {
    /// Create staggered delays for multiple items.
    fn stagger(self, item_count: usize, delay: Duration) -> Vec<Duration>;
}

impl Staggered for Duration {
    /// Create staggered delays given a base duration.
    fn stagger(self, item_count: usize, delay: Duration) -> Vec<Duration> {
        let base_ms = self.as_millis() as f32;
        let delay_ms = delay.as_millis() as f32;

        (0..item_count)
            .map(|i| Duration::from_millis((base_ms + i as f32 * delay_ms) as u64))
            .collect()
    }
}
