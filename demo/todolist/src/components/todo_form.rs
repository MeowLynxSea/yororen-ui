//! yororen-ui Form Component
//!
//! This module demonstrates the standard pattern for building forms in yororen-ui applications.
//! The TodoForm component handles new task creation, providing input fields for task title
//! and category selection, along with a submit button to add the task to the list.
//!
//! ## Form Implementation Pattern
//!
//! Forms in yororen-ui follow a consistent pattern:
//!
//! 1. **Input Handling**: Each input component (text_input, combo_box) has an `on_change` handler
//!    that stores the current value in global state as the user types or selects options.
//! 2. **Validation**: Before performing any action, validate the input to ensure it meets
//!    requirements (e.g., non-empty title for a task).
//! 3. **Action Execution**: When the submit button is clicked (`on_click`), perform the desired
//!    action (create a new todo, update existing data, etc.).
//! 4. **State Reset**: After successful submission, reset form fields to their initial state.
//! 5. **Re-render Trigger**: Call `cx.notify()` to trigger a re-render so the UI reflects the changes.
//!
//! ## Form State Management
//!
//! This component uses global state to persist input values across renders:
//!
//! - `new_todo_title`: Stores the current text input value
//! - `new_todo_category`: Stores the currently selected category
//!
//! This pattern allows the form to maintain its state even when the component re-renders
//! for other reasons.
//!
//! ## Key Components Used
//!
//! - `text_input` - Single-line text input for task title entry
//! - `combo_box` - Dropdown selection for task category
//! - `button` - Action button with `ActionVariantKind::Primary` variant for the main submit action

use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{button, combo_box, text_input, ComboBoxOption};
use yororen_ui::i18n::Translate;
use yororen_ui::theme::ActionVariantKind;

use crate::state::TodoState;
use crate::todo::{Todo, TodoCategory};

/// Form component demonstrating yororen-ui form patterns
pub struct TodoForm;

impl TodoForm {
    /// Standard form render pattern
    pub fn render(cx: &gpui::App, current_category: TodoCategory) -> impl IntoElement {
        let add_placeholder = cx.t("demo.todolist.add_placeholder");
        let add_label = cx.t("demo.todolist.add");

        // Build options for dropdown (common pattern)
        let category_options: Vec<ComboBoxOption> = TodoCategory::all()
            .iter()
            .map(|c| ComboBoxOption::new(c.code(), cx.t(c.key())))
            .collect();

        let category_label = current_category.code().to_string();

        div()
            .flex()
            .items_center()
            .gap(px(12.))
            // Pattern 1: Input with on_change handler
            .child(
                text_input("new-todo")
                    .gap_2()
                    .placeholder(add_placeholder)
                    .on_change(|text, _window, cx| {
                        // Store value in global state for persistence
                        let state = cx.global::<TodoState>();
                        *state.new_todo_title.lock().unwrap() = text.to_string();
                    }),
            )
            // Pattern 2: Dropdown with on_change handler
            .child(
                combo_box("new-category")
                    .gap_2()
                    .value(&category_label)
                    .options(category_options.clone())
                    .on_change(|value, _ev, _window, cx| {
                        let state = cx.global::<TodoState>();
                        if let Some(cat) = TodoCategory::all().into_iter().find(|c| c.code() == value) {
                            *state.new_todo_category.lock().unwrap() = cat;
                        }
                    }),
            )
            // Pattern 3: Action button with on_click handler
            .child(
                button("add-btn")
                    .gap_2()
                    .variant(ActionVariantKind::Primary)  // Use Primary for main action
                    .child(add_label)
                    .on_click(|_ev, _window, cx| {
                        let state = cx.global::<TodoState>();
                        let title = state.new_todo_title.lock().unwrap().clone();
                        if !title.trim().is_empty() {
                            // Perform action
                            let category = state.new_todo_category.lock().unwrap().clone();
                            let todo = Todo::new(title.trim().to_string(), category);
                            state.todos.lock().unwrap().insert(0, todo);

                            // Reset form
                            *state.new_todo_title.lock().unwrap() = String::new();

                            // IMPORTANT: Notify root component to re-render
                            let entity_id = state.notify_entity.lock().unwrap().clone();
                            if let Some(id) = entity_id {
                                cx.notify(id);
                            }
                        }
                    }),
            )
    }
}
