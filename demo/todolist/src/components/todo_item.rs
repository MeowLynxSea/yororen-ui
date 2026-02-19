//! yororen-ui List Item Pattern
//!
//! This component demonstrates two common patterns for rendering list items in yororen-ui.
//!
//! ## Pattern 1: Compact Layout (div-based)
//! Uses basic div with flexbox for simple horizontal layouts.
//! Good for: Toolbars, simple lists, compact views.
//!
//! ## Pattern 2: List Item Layout (list_item-based)
//! Uses the `list_item` component with leading/content/trailing sections.
//! Good for: Complex lists with multiple data points, proper accessibility.
//!
//! ## Key Components Used
//!
//! - `checkbox` - Toggle state (completed/pending)
//! - `icon_button` - Icon-only buttons for actions
//! - `tag` - Category/labels
//! - `list_item` - Structured list item with sections
//! - `label` - Text display

use gpui::prelude::FluentBuilder;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{
    checkbox, icon_button, label, list_item, tag, IconName,
};
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
                        let state = cx.global::<TodoState>();
                        if let Some(t) = state.todos.lock().unwrap().iter_mut().find(|t| t.id == todo_id) {
                            t.completed = !t.completed;
                        }
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
                                let entity_id = {
                                    let state = cx.global::<TodoState>();
                                    if let Some(t) = state.todos.lock().unwrap().iter().find(|t| t.id == todo_id) {
                                        *state.editing_todo.lock().unwrap() = Some(todo_id);
                                        *state.edit_title.lock().unwrap() = t.title.clone();
                                        *state.edit_category.lock().unwrap() = t.category.clone();
                                    }
                                    state.notify_entity.lock().unwrap().clone()
                                };
                                if let Some(entity_id) = entity_id {
                                    cx.notify(entity_id);
                                }
                            }),
                    )
                    .child(
                        icon_button(format!("delete-{}", todo_id))
                            .icon(IconName::Trash)
                            .on_click(move |_ev, _window, cx| {
                                let entity_id = {
                                    let state = cx.global::<TodoState>();
                                    state.todos.lock().unwrap().retain(|t| t.id != todo_id);
                                    state.notify_entity.lock().unwrap().clone()
                                };
                                if let Some(entity_id) = entity_id {
                                    cx.notify(entity_id);
                                }
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
                        let state = cx.global::<TodoState>();
                        if let Some(t) = state.todos.lock().unwrap().iter_mut().find(|t| t.id == todo_id) {
                            t.completed = !t.completed;
                        }
                    }),
            )
            .content(
                div()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        label(&title)
                            .text_size(px(16.))
                    )
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
                                let entity_id = {
                                    let state = cx.global::<TodoState>();
                                    if let Some(t) = state.todos.lock().unwrap().iter().find(|t| t.id == todo_id) {
                                        *state.editing_todo.lock().unwrap() = Some(todo_id);
                                        *state.edit_title.lock().unwrap() = t.title.clone();
                                        *state.edit_category.lock().unwrap() = t.category.clone();
                                    }
                                    state.notify_entity.lock().unwrap().clone()
                                };
                                if let Some(entity_id) = entity_id {
                                    cx.notify(entity_id);
                                }
                            }),
                    )
                    .child(
                        icon_button(format!("delete-btn-{}", todo_id))
                            .icon(IconName::Trash)
                            .on_click(move |_ev, _window, cx| {
                                let entity_id = {
                                    let state = cx.global::<TodoState>();
                                    state.todos.lock().unwrap().retain(|t| t.id != todo_id);
                                    state.notify_entity.lock().unwrap().clone()
                                };
                                if let Some(entity_id) = entity_id {
                                    cx.notify(entity_id);
                                }
                            }),
                    ),
            )
    }
}
