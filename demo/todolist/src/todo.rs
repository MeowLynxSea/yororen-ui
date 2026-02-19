//! yororen-ui Domain Model Pattern
//!
//! This module demonstrates how to structure domain models in yororen-ui applications.
//!
//! ## Domain Model Pattern
//!
//! Keep your domain models (data structures) separate from UI code:
//! - No gpui or yororen-ui imports
//! - Plain Rust structs and enums
//! - Business logic only
//!
//! ## Why Separate Models?
//!
//! Separating models from UI makes code:
//! - Testable: Business logic can be unit tested without UI
//! - Reusable: Models can be used in different contexts
//! - Clean: Clear separation of concerns
//!
//! ## This Pattern in Your App
//!
//! Create a `models.rs` or similar module for your domain types:
//! ```ignore
//! // models.rs
//! pub struct MyEntity { ... }
//! pub enum MyStatus { ... }
//! ```

use uuid::Uuid;

/// Domain entity - represents a todo item
#[derive(Clone, Debug)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub category: TodoCategory,
}

/// Domain enum - represents categories
#[derive(Clone, Debug, PartialEq)]
pub enum TodoCategory {
    Work,
    Personal,
    Shopping,
    Health,
    Other,
}

impl TodoCategory {
    /// Helper method to get all category variants
    pub fn all() -> Vec<TodoCategory> {
        vec![
            TodoCategory::Work,
            TodoCategory::Personal,
            TodoCategory::Shopping,
            TodoCategory::Health,
            TodoCategory::Other,
        ]
    }

    /// Stable category code for state/storage.
    pub fn code(&self) -> &'static str {
        match self {
            TodoCategory::Work => "work",
            TodoCategory::Personal => "personal",
            TodoCategory::Shopping => "shopping",
            TodoCategory::Health => "health",
            TodoCategory::Other => "other",
        }
    }

    /// Translation key for UI label.
    pub fn key(&self) -> &'static str {
        match self {
            TodoCategory::Work => "demo.todolist.categories.work",
            TodoCategory::Personal => "demo.todolist.categories.personal",
            TodoCategory::Shopping => "demo.todolist.categories.shopping",
            TodoCategory::Health => "demo.todolist.categories.health",
            TodoCategory::Other => "demo.todolist.categories.other",
        }
    }
}

impl Todo {
    /// Factory method for creating new todos
    pub fn new(title: String, category: TodoCategory) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            completed: false,
            category,
        }
    }
}
