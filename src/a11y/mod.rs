//! Accessibility (A11y) module for Yororen UI.
//!
//! This module provides accessibility utilities including:
//! - ARIA role and attribute definitions
//! - Focus management components (FocusTrap)
//! - Keyboard navigation helpers

mod aria;
mod focus_trap;

pub use aria::*;
pub use focus_trap::*;
