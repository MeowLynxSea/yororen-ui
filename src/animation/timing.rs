//! Time and progress helpers for animations.
//!
//! This module is intentionally gpui-agnostic: it works with `std::time::Duration`
//! and normalized progress values.

use std::time::Duration;

/// Clamp a value to the `[0.0, 1.0]` range.
#[inline]
pub fn clamp01(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// Convert an elapsed duration into linear progress in `[0.0, 1.0]`.
///
/// - Returns `1.0` when `duration` is zero.
/// - Clamps the result to `[0.0, 1.0]`.
#[inline]
pub fn progress_from_elapsed(elapsed: Duration, duration: Duration) -> f32 {
    if duration.is_zero() {
        return 1.0;
    }

    let elapsed_secs = elapsed.as_secs_f32();
    let duration_secs = duration.as_secs_f32();
    if duration_secs <= 0.0 {
        return 1.0;
    }
    clamp01(elapsed_secs / duration_secs)
}

/// Convert an overall progress in `[0.0, 1.0]` into per-item progress.
///
/// Returns `(index, progress)` where:
/// - `index` is the active item index
/// - `progress` is the normalized progress within that item
///
/// If `durations` is empty, returns `(0, 1.0)`.
/// If total duration is zero, returns the last index with progress `1.0`.
pub fn sequence_progress(durations: &[Duration], total_progress: f32) -> (usize, f32) {
    if durations.is_empty() {
        return (0, 1.0);
    }

    let total_ms: u128 = durations.iter().map(|d| d.as_millis()).sum();
    if total_ms == 0 {
        return (durations.len() - 1, 1.0);
    }

    let total_progress = clamp01(total_progress);
    let current_ms = (total_progress * (total_ms as f32)).max(0.0);

    let mut accumulated_ms = 0f32;
    for (index, duration) in durations.iter().enumerate() {
        let duration_ms = duration.as_millis() as f32;
        if duration_ms <= 0.0 {
            continue;
        }

        if current_ms < accumulated_ms + duration_ms {
            let item_progress = (current_ms - accumulated_ms) / duration_ms;
            return (index, clamp01(item_progress));
        }

        accumulated_ms += duration_ms;
    }

    (durations.len(), 1.0)
}

/// Convert overall progress in `[0.0, 1.0]` into per-item progress for parallel animations.
///
/// Each item progresses based on its own duration relative to the longest duration.
/// If `durations` is empty or `index` is out of bounds, returns `1.0`.
pub fn parallel_progress(durations: &[Duration], total_progress: f32, index: usize) -> f32 {
    if durations.is_empty() || index >= durations.len() {
        return 1.0;
    }

    let max_ms = durations.iter().map(|d| d.as_millis()).max().unwrap_or(0);
    if max_ms == 0 {
        return 1.0;
    }

    let item_ms = durations[index].as_millis();
    if item_ms == 0 {
        return 1.0;
    }

    let ratio = (item_ms as f32) / (max_ms as f32);
    if ratio <= 0.0 {
        return 1.0;
    }

    let total_progress = clamp01(total_progress);
    clamp01(total_progress / ratio)
}
