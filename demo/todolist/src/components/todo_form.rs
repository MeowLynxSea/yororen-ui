//! yororen-ui Form Pattern
//!
//! This component demonstrates the standard pattern for building forms in yororen-ui.
//!
//! ## Form Pattern
//!
//! Forms in yororen-ui typically:
//! 1. Store input values in global state (via `on_change` handlers)
//! 2. Validate input before submission
//! 3. Perform action on submit (via `on_click`)
//! 4. Trigger re-render after state change
//!
//! ## Key Components Used
//!
//! - `text_input` - Single-line text input
//! - `combo_box` - Dropdown selection
//! - `button` - Action buttons (use `ActionVariantKind::Primary` for main action)

use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{button, combo_box, text_input, ComboBoxOption};
use yororen_ui::theme::ActionVariantKind;

use crate::state::TodoState;
use crate::todo::{Todo, TodoCategory};

/// Form component demonstrating yororen-ui form patterns
pub struct TodoForm;

impl TodoForm {
    /// Standard form render pattern
    pub fn render(current_category: TodoCategory) -> impl IntoElement {
        // Build options for dropdown (common pattern)
        let category_options: Vec<ComboBoxOption> = TodoCategory::all()
            .iter()
            .map(|c| ComboBoxOption::new(c.label(), c.label()))
            .collect();

        let category_label = current_category.label().to_string();

        div()
            .flex()
            .items_center()
            .gap(px(12.))
            // Pattern 1: Input with on_change handler
            .child(
                text_input("new-todo")
                    .gap_2()
                    .placeholder("Add new task...")
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
                        if let Some(cat) = TodoCategory::all().into_iter().find(|c| c.label() == value) {
                            *state.new_todo_category.lock().unwrap() = cat;
                        }
                    }),
            )
            // Pattern 3: Action button with on_click handler
            .child(
                button("add-btn")
                    .gap_2()
                    .variant(ActionVariantKind::Primary)  // Use Primary for main action
                    .child("Add")
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
