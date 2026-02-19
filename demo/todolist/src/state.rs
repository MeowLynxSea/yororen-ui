//! yororen-ui Global State Management
//!
//! This module demonstrates the recommended pattern for managing application state in yororen-ui/gpui applications.
//! Global state allows components throughout the application to share data without complex prop drilling.
//!
//! ## Why Arc<Mutex<T>>?
//!
//! In yororen-ui applications, components are often rendered from different contexts (closures and callbacks).
//! Standard Rust ownership rules don't allow multiple parts of the code to mutate the same data simultaneously.
//! The `Arc<Mutex<T>>` pattern solves this architectural challenge:
//!
//! - **Arc** (Atomic Reference Counted): Enables multiple owners to share access to the same data.
//!   When an Arc is cloned, it increments an internal reference count; when dropped, it decrements the count.
//!   The data is only deallocated when the count reaches zero.
//! - **Mutex** (Mutual Exclusion): Ensures thread-safe access by allowing only one thread (or context) to
//!   mutate the data at a time. Other threads must wait until the lock is released.
//!
//! ## State Structure Design
//!
//! Global state is typically organized into several categories of data:
//!
//! - **Application Data**: The core business data (e.g., `todos` - the list of todo items)
//! - **UI State**: State that controls how the UI is displayed (e.g., `compact_mode`, `editing_todo`)
//! - **Form State**: Temporary state for form inputs (e.g., `edit_title`, `edit_category`, `new_todo_title`)
//! - **System State**: Infrastructure state like the notification entity ID
//!
//! ## State Update Flow
//!
//! ```ignore
//! 1. Component reads state: let value = *state.field.lock().unwrap();
//! 2. Component modifies state: *state.field.lock().unwrap() = new_value;
//! 3. Component triggers re-render: cx.notify(entity_id);
//! 4. gpui re-renders the component that owns the entity
//! ```
//!
//! ## Implementing Global State
//!
//! To add global state to your yororen-ui application:
//!
//! 1. Define a struct containing `Arc<Mutex<T>>` fields for each piece of state
//! 2. Implement the `Clone` trait (can be derived automatically)
//! 3. Implement `Default` to provide initial state values
//! 4. Implement the `Global` trait from gpui to make the type accessible via `cx.global::<T>()`
//! 5. Register the state in main() using `cx.set_global(YourState::default())`
//!
//! ## Thread Safety Considerations
//!
//! When working with global state, be mindful of:
//! - **Lock Ordering**: Always acquire and release locks in a consistent order to prevent deadlocks
//! - **Minimize Lock Duration**: Keep critical sections as short as possible
//! - **Clone Before Lock Release**: Extract needed data from locks before releasing them

use gpui::{EntityId, Global};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::todo::{Todo, TodoCategory};

/// Standard yororen-ui global state struct
///
/// All fields follow the Arc<Mutex<T>> pattern for safe shared mutation.
/// This struct is accessible from any component via `cx.global::<TodoState>()`.
pub struct TodoState {
    // Application data
    pub todos: Arc<Mutex<Vec<Todo>>>,
    pub search_query: Arc<Mutex<String>>,
    pub selected_category: Arc<Mutex<Option<TodoCategory>>>,

    // UI state
    pub compact_mode: Arc<Mutex<bool>>,
    pub editing_todo: Arc<Mutex<Option<Uuid>>>,

    // Form state
    pub edit_title: Arc<Mutex<String>>,
    pub edit_category: Arc<Mutex<TodoCategory>>,
    /// When true, the edit modal should initialize its input fields from the edit_* buffers.
    /// This is set when opening the modal (or switching the edited todo), and cleared after init.
    pub edit_needs_init: Arc<Mutex<bool>>,
    pub new_todo_category: Arc<Mutex<TodoCategory>>,
    pub new_todo_title: Arc<Mutex<String>>,
    pub clear_input_flag: Arc<Mutex<bool>>,

    // Notification system
    // Stores entity_id of the component that should receive re-render notifications
    pub notify_entity: Arc<Mutex<Option<EntityId>>>,
}

/// Required for sharing state across components
impl Clone for TodoState {
    fn clone(&self) -> Self {
        Self {
            todos: self.todos.clone(),
            search_query: self.search_query.clone(),
            selected_category: self.selected_category.clone(),
            compact_mode: self.compact_mode.clone(),
            editing_todo: self.editing_todo.clone(),
            edit_title: self.edit_title.clone(),
            edit_category: self.edit_category.clone(),
            edit_needs_init: self.edit_needs_init.clone(),
            new_todo_category: self.new_todo_category.clone(),
            new_todo_title: self.new_todo_title.clone(),
            clear_input_flag: self.clear_input_flag.clone(),
            notify_entity: self.notify_entity.clone(),
        }
    }
}

/// Provides initial state when app starts
impl Default for TodoState {
    fn default() -> Self {
        // Demo data - replace with your app's initial state
        let mut todos = Vec::new();
        todos.push(Todo::new(
            "Complete project report".to_string(),
            TodoCategory::Work,
        ));
        todos.push(Todo::new(
            "Buy groceries".to_string(),
            TodoCategory::Shopping,
        ));
        todos.push(Todo::new("Go to gym".to_string(), TodoCategory::Health));
        todos[0].completed = true;

        Self {
            todos: Arc::new(Mutex::new(todos)),
            search_query: Arc::new(Mutex::new(String::new())),
            selected_category: Arc::new(Mutex::new(None)),
            compact_mode: Arc::new(Mutex::new(false)),
            editing_todo: Arc::new(Mutex::new(None)),
            edit_title: Arc::new(Mutex::new(String::new())),
            edit_category: Arc::new(Mutex::new(TodoCategory::Other)),
            edit_needs_init: Arc::new(Mutex::new(false)),
            new_todo_category: Arc::new(Mutex::new(TodoCategory::Personal)),
            new_todo_title: Arc::new(Mutex::new(String::new())),
            clear_input_flag: Arc::new(Mutex::new(false)),
            notify_entity: Arc::new(Mutex::new(None)),
        }
    }
}

/// REQUIRED: Makes this type available as global state
///
/// Only types implementing `Global` can be accessed via `cx.global::<T>()`
impl Global for TodoState {}
