//! Easing functions for animations.
//!
//! Provides a collection of easing functions for smooth animations.
//! Each function takes a normalized time value (0.0 to 1.0) and returns
//! the eased progress value.

/// A function that maps linear progress to eased progress.
pub type EasingFn = fn(f32) -> f32;

// ============================================================================
// Linear
// ============================================================================

/// Linear easing (no easing).
pub const fn ease_linear(t: f32) -> f32 {
    t
}

// ============================================================================
// Sine
// ============================================================================

/// Ease in with sine function.
pub fn ease_in_sine(t: f32) -> f32 {
    1.0 - (t * std::f32::consts::FRAC_PI_2).cos()
}

/// Ease out with sine function.
pub fn ease_out_sine(t: f32) -> f32 {
    (t * std::f32::consts::FRAC_PI_2).sin()
}

/// Ease in-out with sine function.
pub fn ease_in_out_sine(t: f32) -> f32 {
    -(t * std::f32::consts::PI).cos() / 2.0 + 0.5
}

// ============================================================================
// Quadratic (Power 2)
// ============================================================================

/// Ease in with quadratic function (t^2).
pub const fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Ease out with quadratic function (t^2).
pub const fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Ease in-out with quadratic function (t^2).
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (2.0 - 2.0 * t) * (2.0 - 2.0 * t) / 2.0
    }
}

// ============================================================================
// Cubic (Power 3)
// ============================================================================

/// Ease in with cubic function (t^3).
pub const fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// Ease out with cubic function (t^3).
pub const fn ease_out_cubic(t: f32) -> f32 {
    let t = 1.0 - t;
    1.0 - t * t * t
}

/// Ease in-out with cubic function (t^3).
pub const fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t = 2.0 * t - 2.0;
        (t * t * t + 2.0) / 2.0
    }
}

// ============================================================================
// Quartic (Power 4)
// ============================================================================

/// Ease in with quartic function (t^4).
pub const fn ease_in_quart(t: f32) -> f32 {
    t * t * t * t
}

/// Ease out with quartic function (t^4).
pub const fn ease_out_quart(t: f32) -> f32 {
    let t = 1.0 - t;
    1.0 - t * t * t * t
}

/// Ease in-out with quartic function (t^4).
#[allow(dead_code)]
pub const fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let t = 1.0 - t;
        1.0 - 8.0 * t * t * t * t
    }
}

// ============================================================================
// Quintic (Power 5)
// ============================================================================

/// Ease in with quintic function (t^5).
pub const fn ease_in_quint(t: f32) -> f32 {
    t * t * t * t * t
}

/// Ease out with quintic function (t^5).
pub const fn ease_out_quint(t: f32) -> f32 {
    let t = 1.0 - t;
    1.0 - t * t * t * t * t
}

/// Ease in-out with quintic function (t^5).
pub const fn ease_in_out_quint(t: f32) -> f32 {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        let t = 2.0 * t - 2.0;
        (t * t * t * t * t + 2.0) / 2.0
    }
}

// ============================================================================
// Exponential
// ============================================================================

/// Ease in with exponential function (2^10*(t-1)).
pub fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0_f32.powf(10.0 * t - 10.0)
    }
}

/// Ease out with exponential function (-2^(-10*t) + 1).
pub fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * t)
    }
}

/// Ease in-out with exponential function.
pub fn ease_in_out_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        2.0_f32.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
    }
}

// ============================================================================
// Circular
// ============================================================================

/// Ease in with circular function.
pub fn ease_in_circ(t: f32) -> f32 {
    1.0 - (1.0 - t * t).sqrt()
}

/// Ease out with circular function.
pub fn ease_out_circ(t: f32) -> f32 {
    let t = t - 1.0;
    (1.0 - t * t).sqrt()
}

/// Ease in-out with circular function.
#[allow(dead_code)]
pub fn ease_in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - (1.0 - 4.0 * t * t).sqrt()) / 2.0
    } else {
        ((4.0 * t * t - 4.0 * t + 1.0).sqrt() + 1.0) / 2.0
    }
}

// ============================================================================
// Back (Overshoot)
// ============================================================================

/// Ease in with back/overshoot effect.
#[allow(dead_code)]
pub fn ease_in_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    c3 * t * t * t - c1 * t * t
}

/// Ease out with back/overshoot effect.
pub fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// Ease in-out with back/overshoot effect.
pub fn ease_in_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c2 = c1 * 1.525;
    if t < 0.5 {
        ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
    } else {
        ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
    }
}

// ============================================================================
// Elastic (Spring-like)
// ============================================================================

/// Ease in with elastic/spring effect.
pub fn ease_in_elastic(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * std::f32::consts::PI) / 3.0;
        -(2.0_f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
    }
}

/// Ease out with elastic/spring effect.
pub fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * std::f32::consts::PI) / 3.0;
        2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
    }
}

/// Ease in-out with elastic/spring effect.
pub fn ease_in_out_elastic(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c5 = (2.0 * std::f32::consts::PI) / 4.5;
        if t < 0.5 {
            -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
        } else {
            (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
        }
    }
}

// ============================================================================
// Bounce
// ============================================================================

/// Ease in with bounce effect.
pub fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

/// Ease out with bounce effect.
pub fn ease_out_bounce(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;

    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

/// Ease in-out with bounce effect.
pub fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

// ============================================================================
// Basic easing (matching gpui)
// ============================================================================

/// Ease in (simple quadratic).
pub const fn ease_in(t: f32) -> f32 {
    t * t
}

/// Ease out (simple quadratic).
pub const fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Ease in-out (simple quadratic).
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (2.0 - 2.0 * t) * (2.0 - 2.0 * t) / 2.0
    }
}
