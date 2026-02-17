//! yororen-ui Simple State Pattern
//!
//! This module demonstrates the **minimal** state pattern for yororen-ui applications.
//! Unlike the todolist demo which has complex state, this shows the simplest form:
//! just an Arc<Mutex<T>> wrapping a primitive type.
//!
//! ## Key Concepts
//!
//! - **Arc<Mutex<T>>**: Allows shared mutable access across components
//!   - Arc: Multiple components can own the data
//!   - Mutex: Only one component can modify at a time
//!
//! ## State Update Flow
//!
//! ```ignore
//! 1. Read: let value = *state.counter.lock().unwrap();
//! 2. Modify: *state.counter.lock().unwrap() = value + 1;
//! 3. Notify: cx.notify(entity_id);  // Trigger re-render
//! ```

use gpui::{EntityId, Global};
use std::sync::{Arc, Mutex};

/// Simple counter state - just a number!
///
/// This is the minimal state structure. For more complex apps,
/// add more Arc<Mutex<T>> fields as needed.
pub struct CounterState {
    /// The current counter value
    pub counter: Arc<Mutex<i32>>,

    /// Entity ID for triggering re-renders
    /// Stored when the component is created, used to notify it of changes
    pub notify_entity: Arc<Mutex<Option<EntityId>>>,
}

/// Required for sharing state across components
impl Clone for CounterState {
    fn clone(&self) -> Self {
        Self {
            counter: self.counter.clone(),
            notify_entity: self.notify_entity.clone(),
        }
    }
}

/// Initial state when app starts
impl Default for CounterState {
    fn default() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            notify_entity: Arc::new(Mutex::new(None)),
        }
    }
}

/// Required: Makes this type accessible as global state
impl Global for CounterState {}
