//! Animation module for UI components.
//!
//! This module provides a unified animation system with preset animations,
//! easing functions, and orchestration capabilities.

mod config;
mod easing;
mod helpers;
mod orchestrator;
mod preset;
mod timing;

pub mod constants;

/// Convenience re-export of global duration constants.
pub use constants::duration;

pub use config::{AnimationConfig, AnimationState};
pub use easing::{
    EasingFn, clamp_easing, ease_in, ease_in_back, ease_in_back_clamped, ease_in_bounce,
    ease_in_bounce_clamped, ease_in_circ, ease_in_circ_clamped, ease_in_cubic,
    ease_in_cubic_clamped, ease_in_elastic, ease_in_elastic_clamped, ease_in_expo,
    ease_in_expo_clamped, ease_in_out, ease_in_out_back, ease_in_out_back_clamped,
    ease_in_out_bounce, ease_in_out_bounce_clamped, ease_in_out_circ, ease_in_out_circ_clamped,
    ease_in_out_clamped, ease_in_out_cubic, ease_in_out_cubic_clamped, ease_in_out_elastic,
    ease_in_out_elastic_clamped, ease_in_out_expo, ease_in_out_expo_clamped, ease_in_out_quad,
    ease_in_out_quad_clamped, ease_in_out_quart, ease_in_out_quart_clamped, ease_in_out_quint,
    ease_in_out_quint_clamped, ease_in_out_sine, ease_in_out_sine_clamped, ease_in_quad,
    ease_in_quad_clamped, ease_in_quart, ease_in_quart_clamped, ease_in_quint,
    ease_in_quint_clamped, ease_in_sine, ease_in_sine_clamped, ease_linear, ease_out,
    ease_out_back, ease_out_back_clamped, ease_out_bounce, ease_out_bounce_clamped, ease_out_circ,
    ease_out_circ_clamped, ease_out_cubic, ease_out_cubic_clamped, ease_out_elastic,
    ease_out_elastic_clamped, ease_out_expo, ease_out_expo_clamped, ease_out_quad,
    ease_out_quad_clamped, ease_out_quart, ease_out_quart_clamped, ease_out_quint,
    ease_out_quint_clamped, ease_out_sine, ease_out_sine_clamped,
};
pub use helpers::{
    AnimateExt, SlideDirection as HelpersSlideDirection, animation_id, lerp, lerp_color,
};
pub use orchestrator::{AnimationParallel, AnimationSequence, Staggered, parallel, sequence};
pub use orchestrator::{Orchestration, TrackId};
pub use preset::{
    AnimationType, BounceIn, BounceOut, ElasticIn, ElasticOut, FadeIn, FadeOut, PresetAnimation,
    ScaleIn, ScaleOut, SlideDirection as PresetSlideDirection, bounce_in_down, bounce_in_left,
    bounce_in_right, bounce_in_up, bounce_out_to, elastic_scale_in, elastic_scale_out,
    fade_scale_in, fade_scale_out, fade_slide_in, fade_slide_in_down, fade_slide_in_from,
    fade_slide_in_left, fade_slide_in_right, fade_slide_in_up, fade_slide_out, fade_slide_out_to,
    preset_duration, pulse,
};

pub use timing::{clamp01, parallel_progress, progress_from_elapsed, sequence_progress};
