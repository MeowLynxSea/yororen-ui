//! Input module - shared code for text input components.
//!
//! This module contains common functionality used by TextInput, TextArea,
//! and PasswordInput components to reduce code duplication.

pub mod actions;

// Re-export the action_handler macro for use in input components
pub use crate::action_handler;
