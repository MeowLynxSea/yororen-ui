use gpui::prelude::FluentBuilder;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{
    checkbox, icon_button, label, list_item, tag, IconName,
};

use crate::state::TodoState;
use crate::todo::Todo;

pub struct TodoItem;

impl TodoItem {
    pub fn render(todo: &Todo, compact_mode: bool) -> AnyElement {
        if compact_mode {
            Self::render_compact(todo).into_any_element()
        } else {
            Self::render_normal(todo).into_any_element()
        }
    }

    fn render_compact(todo: &Todo) -> impl IntoElement {
        let todo_id = todo.id;
        let title = todo.title.clone();
        let category_label = todo.category.label().to_string();
        let completed = todo.completed;

        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .p(px(8.))
            .rounded(px(4.))
            .when(completed, |this| this.opacity(0.6))
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
            .child(tag(&category_label).selected(true))
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

    fn render_normal(todo: &Todo) -> impl IntoElement {
        let todo_id = todo.id;
        let title = todo.title.clone();
        let category_label = todo.category.label().to_string();
        let completed = todo.completed;

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
                            .child(tag(&category_label).selected(true)),
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
