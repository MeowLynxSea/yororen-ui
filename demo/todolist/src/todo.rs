//! yororen-ui Domain Model Layer
//!
//! This module demonstrates how to structure domain models in yororen-ui applications.
//! Domain models represent the core business concepts and are kept separate from UI implementation details.
//!
//! ## Domain Model Pattern
//!
//! Domain models in yororen-ui applications should be kept separate from UI code:
//!
//! - **No UI Dependencies**: Do not import gpui or yororen-ui modules in domain model files
//! - **Plain Rust Types**: Use standard Rust structs and enums
//! - **Business Logic Only**: Include only data structures and their associated methods
//!
//! ## Benefits of Separating Models
//!
//! Keeping domain models separate from UI code provides several advantages:
//!
//! - **Testability**: Business logic can be unit tested without requiring a UI environment
//! - **Reusability**: Models can be used in different contexts (e.g., backend services, CLI tools)
//! - **Clean Architecture**: Clear separation of concerns between data and presentation
//! - **Maintainability**: Changes to UI don't affect business logic, and vice versa
//!
//! ## Implementing Domain Models
//!
//! Create a dedicated module (often named `models.rs` or matching the domain concept) for your types:
//! ```ignore
//! // models.rs or todo.rs
//! pub struct MyEntity { ... }
//! pub enum MyStatus { ... }
//! impl MyEntity { ... }
//! ```
//!
//! ## This Module's Models
//!
//! This module defines:
//!
//! - **Todo**: The main domain entity representing a task item with title, completion status, and category
//! - **TodoCategory**: An enum representing the different categories a todo can belong to (Work, Personal, Shopping, Health, Other)
//!
//! The TodoCategory enum provides helper methods for working with categories:
//! - `all()`: Returns a vector of all category variants
//! - `code()`: Returns a stable string code suitable for storage or comparison
//! - `key()`: Returns the i18n translation key for displaying the category name

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
