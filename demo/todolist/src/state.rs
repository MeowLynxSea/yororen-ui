//! yororen-ui Global State Pattern
//!
//! This module demonstrates the **recommended pattern** for managing application state
//! in yororen-ui/gpui applications.
//!
//! ## Why This Pattern?
//!
//! In yororen-ui, components are often rendered from different contexts (closures).
//! Standard Rust ownership doesn't allow multiple parts of the code to mutate the same data.
//! The `Arc<Mutex<T>>` pattern solves this:
//!
//! - **Arc** (Atomic Reference Counted): Allows multiple owners to share data
//! - **Mutex** (Mutual Exclusion): Ensures only one part can mutate at a time
//!
//! ## State Update Flow (Important!)
//!
//! ```ignore
//! 1. Component reads state: let value = *state.field.lock().unwrap();
//! 2. Component modifies state: *state.field.lock().unwrap() = new_value;
//! 3. Component triggers re-render: cx.notify(entity_id);
//! 4. gpui re-renders the component that owns the entity
//! ```
//!
//! ## This Pattern in Your App
//!
//! To add global state to your yororen-ui application:
//! 1. Define a struct with `Arc<Mutex<T>>` fields
//! 2. Implement `Clone` (automatically via derive or manually)
//! 3. Implement `Default` for initial state
//! 4. Implement `Global` trait
//! 5. Set via `cx.set_global(YourState::default())` in main()

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
