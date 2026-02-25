//! yororen-ui List Item Component
//!
//! This module demonstrates two common patterns for rendering list items in yororen-ui applications.
//! The TodoItem component renders individual todo entries and supports two view modes: compact and detailed.
//!
//! ## View Mode Patterns
//!
//! This component implements two different rendering approaches:
//!
//! ### Pattern 1: Compact Layout (div-based)
//!
//! Uses basic HTML div elements with flexbox for simple horizontal layouts.
//! This pattern is ideal for:
//! - Toolbars and control bars
//! - Simple lists with minimal information
//! - Views where space is at a premium
//! - Scrolling lists with many items
//!
//! The compact view displays the checkbox, title, category tag, and action icons in a single row.
//!
//! ### Pattern 2: List Item Layout (list_item-based)
//!
//! Uses the `list_item` component which provides a structured layout with three distinct sections:
//! - **leading**: Primary action area on the left (checkbox for completion toggle)
//! - **content**: Main content area in the center (title and category)
//! - **trailing**: Secondary actions on the right (edit and delete buttons)
//!
//! This pattern is ideal for:
//! - Complex lists with multiple data points
//! - Items requiring proper accessibility semantics
//! - Views where visual hierarchy is important
//! - List items that need consistent spacing and alignment
//!
//! ## Conditional Rendering
//!
//! The component switches between view modes based on the `compact_mode` parameter:
//! - `true`: Renders the compact div-based layout
//! - `false`: Renders the full list_item layout with structured sections
//!
//! ## Item Actions
//!
//! Each todo item provides two action buttons:
//!
//! - **Edit Button**: Opens the modal dialog for editing the task, populating the edit buffers
//!   with the current task values and setting the `editing_todo` state
//! - **Delete Button**: Removes the task from the global state, triggering a re-render
//!
//! ## Key Components Used
//!
//! - `checkbox` - Toggle button for completed/pending state
//! - `icon_button` - Icon-only buttons for edit and delete actions
//! - `tag` - Visual label displaying the task category
//! - `list_item` - Structured list item container with leading/content/trailing sections
//! - `label` - Text display for task title

use gpui::prelude::FluentBuilder;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{IconName, checkbox, icon_button, label, list_item, tag};
use yororen_ui::i18n::Translate;

use crate::state::TodoState;
use crate::todo::Todo;

/// Demonstrates two list item rendering patterns
pub struct TodoItem;

impl TodoItem {
    /// Renders a todo item - demonstrates conditional rendering pattern
    pub fn render(cx: &gpui::App, todo: &Todo, compact_mode: bool) -> AnyElement {
        if compact_mode {
            Self::render_compact(cx, todo).into_any_element()
        } else {
            Self::render_normal(cx, todo).into_any_element()
        }
    }

    /// Pattern 1: Compact div-based layout
    fn render_compact(cx: &gpui::App, todo: &Todo) -> impl IntoElement {
        let todo_id = todo.id;
        let title = todo.title.clone();
        let category_label = cx.t(todo.category.key());
        let completed = todo.completed;

        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .p(px(8.))
            .rounded(px(4.))
            // Conditional styling with .when()
            .when(completed, |this| this.opacity(0.6))
            // Checkbox with on_toggle handler
            .child(
                checkbox(format!("todo-{}", todo_id))
                    .checked(completed)
                    .on_toggle(move |_, _, _window, cx| {
                        let model = cx.global::<TodoState>().model.clone();
                        model.update(cx, |model, cx| {
                            if let Some(t) = model.todos.iter_mut().find(|t| t.id == todo_id) {
                                t.completed = !t.completed;
                                cx.notify();
                            }
                        });
                    }),
            )
            .child(label(&title))
            .child(tag(category_label).selected(true))
            // Icon buttons for actions
            .child(
                div()
                    .flex()
                    .gap(px(4.))
                    .child(
                        icon_button(format!("edit-{}", todo_id))
                            .icon(IconName::Pencil)
                            .on_click(move |_ev, _window, cx| {
                                let model = cx.global::<TodoState>().model.clone();
                                model.update(cx, |model, cx| {
                                    if let Some(t) = model.todos.iter().find(|t| t.id == todo_id) {
                                        model.editing_todo = Some(todo_id);
                                        model.edit_title = t.title.clone();
                                        model.edit_category = t.category.clone();
                                        model.edit_needs_init = true;
                                        cx.notify();
                                    }
                                });
                            }),
                    )
                    .child(
                        icon_button(format!("delete-{}", todo_id))
                            .icon(IconName::Trash)
                            .on_click(move |_ev, _window, cx| {
                                let model = cx.global::<TodoState>().model.clone();
                                model.update(cx, |model, cx| {
                                    model.todos.retain(|t| t.id != todo_id);
                                    cx.notify();
                                });
                            }),
                    ),
            )
    }

    /// Pattern 2: Full list_item with leading/content/trailing
    fn render_normal(cx: &gpui::App, todo: &Todo) -> impl IntoElement {
        let todo_id = todo.id;
        let title = todo.title.clone();
        let category_label = cx.t(todo.category.key());
        let completed = todo.completed;

        // list_item provides structured layout
        // - leading: actions on the left (checkbox)
        // - content: main content (title, category)
        // - trailing: actions on the right (edit, delete)
        list_item()
            .id(format!("todo-item-{}", todo_id))
            .leading(
                checkbox(format!("todo-check-{}", todo_id))
                    .checked(completed)
                    .on_toggle(move |_, _, _window, cx| {
                        let model = cx.global::<TodoState>().model.clone();
                        model.update(cx, |model, cx| {
                            if let Some(t) = model.todos.iter_mut().find(|t| t.id == todo_id) {
                                t.completed = !t.completed;
                                cx.notify();
                            }
                        });
                    }),
            )
            .content(
                div()
                    .flex_col()
                    .gap(px(4.))
                    .child(label(&title).text_size(px(16.)))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(tag(category_label).selected(true)),
                    ),
            )
            .trailing(
                div()
                    .flex()
                    .gap(px(4.))
                    .child(
                        icon_button(format!("edit-btn-{}", todo_id))
                            .icon(IconName::Pencil)
                            .on_click(move |_ev, _window, cx| {
                                let model = cx.global::<TodoState>().model.clone();
                                model.update(cx, |model, cx| {
                                    if let Some(t) = model.todos.iter().find(|t| t.id == todo_id) {
                                        model.editing_todo = Some(todo_id);
                                        model.edit_title = t.title.clone();
                                        model.edit_category = t.category.clone();
                                        model.edit_needs_init = true;
                                        cx.notify();
                                    }
                                });
                            }),
                    )
                    .child(
                        icon_button(format!("delete-btn-{}", todo_id))
                            .icon(IconName::Trash)
                            .on_click(move |_ev, _window, cx| {
                                let model = cx.global::<TodoState>().model.clone();
                                model.update(cx, |model, cx| {
                                    model.todos.retain(|t| t.id != todo_id);
                                    cx.notify();
                                });
                            }),
                    ),
            )
    }
}
