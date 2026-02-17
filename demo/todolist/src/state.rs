use gpui::{EntityId, Global};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::todo::{Todo, TodoCategory};

pub struct TodoState {
    pub todos: Arc<Mutex<Vec<Todo>>>,
    pub search_query: Arc<Mutex<String>>,
    pub selected_category: Arc<Mutex<Option<TodoCategory>>>,
    pub compact_mode: Arc<Mutex<bool>>,
    pub editing_todo: Arc<Mutex<Option<Uuid>>>,
    pub edit_title: Arc<Mutex<String>>,
    pub edit_category: Arc<Mutex<TodoCategory>>,
    pub new_todo_category: Arc<Mutex<TodoCategory>>,
    pub new_todo_title: Arc<Mutex<String>>,
    pub clear_input_flag: Arc<Mutex<bool>>,
    pub notify_entity: Arc<Mutex<Option<EntityId>>>,
}

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
            new_todo_category: self.new_todo_category.clone(),
            new_todo_title: self.new_todo_title.clone(),
            clear_input_flag: self.clear_input_flag.clone(),
            notify_entity: self.notify_entity.clone(),
        }
    }
}

impl Default for TodoState {
    fn default() -> Self {
        let mut todos = Vec::new();
        todos.push(Todo::new("完成项目报告".to_string(), TodoCategory::Work));
        todos.push(Todo::new("购买生活用品".to_string(), TodoCategory::Shopping));
        todos.push(Todo::new("健身锻炼".to_string(), TodoCategory::Health));
        todos[0].completed = true;

        Self {
            todos: Arc::new(Mutex::new(todos)),
            search_query: Arc::new(Mutex::new(String::new())),
            selected_category: Arc::new(Mutex::new(None)),
            compact_mode: Arc::new(Mutex::new(false)),
            editing_todo: Arc::new(Mutex::new(None)),
            edit_title: Arc::new(Mutex::new(String::new())),
            edit_category: Arc::new(Mutex::new(TodoCategory::Other)),
            new_todo_category: Arc::new(Mutex::new(TodoCategory::Personal)),
            new_todo_title: Arc::new(Mutex::new(String::new())),
            clear_input_flag: Arc::new(Mutex::new(false)),
            notify_entity: Arc::new(Mutex::new(None)),
        }
    }
}

impl Global for TodoState {}
