//! Library-wide constants.
//!
//! Note: animation/timing constants live in `crate::animation::constants`. This
//! module re-exports them for backward compatibility.

pub use crate::animation::constants::*;

/// Backwards-compatible export path for older code.
pub mod animation {
    pub use crate::animation::constants::duration::*;
}
