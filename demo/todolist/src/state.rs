//! yororen-ui Global State Management (Entity-based)
//!
//! This demo stores app state in a `gpui::Entity<TodoModel>`.
//!
//! Why:
//! - GPUI tracks entity reads during render and invalidates windows efficiently.
//! - No `Arc<Mutex<...>>` and no manual storage of a root `EntityId` is required.
//!
//! Important: In `gpui-ce 0.3`, mutating an entity does not automatically trigger a redraw.
//! Call `cx.notify()` inside `Entity::update` after changing fields.

use gpui::{App, AppContext, Entity, Global};
use uuid::Uuid;

use crate::todo::{Todo, TodoCategory};

#[derive(Clone)]
pub struct TodoState {
    pub model: Entity<TodoModel>,
}

impl TodoState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            model: cx.new(|_| TodoModel::default()),
        }
    }
}

impl Global for TodoState {}

pub struct TodoModel {
    // Application data
    pub todos: Vec<Todo>,
    pub search_query: String,
    pub selected_category: Option<TodoCategory>,

    // UI state
    pub compact_mode: bool,
    pub editing_todo: Option<Uuid>,

    // Form state
    pub edit_title: String,
    pub edit_category: TodoCategory,
    pub edit_needs_init: bool,
    pub new_todo_category: TodoCategory,
    pub new_todo_title: String,
}

impl Default for TodoModel {
    fn default() -> Self {
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
            todos,
            search_query: String::new(),
            selected_category: None,
            compact_mode: false,
            editing_todo: None,
            edit_title: String::new(),
            edit_category: TodoCategory::Other,
            edit_needs_init: false,
            new_todo_category: TodoCategory::Personal,
            new_todo_title: String::new(),
        }
    }
}
