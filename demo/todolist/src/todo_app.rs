use gpui::{
    prelude::FluentBuilder,
    Context, IntoElement, ParentElement,
    Render, Styled, Window, div, px,
};
use yororen_ui::theme::ActiveTheme;

use crate::components;
use crate::todo::Todo;
use crate::state::TodoState;

pub struct TodoApp;

impl TodoApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.global::<TodoState>();
        *state.notify_entity.lock().unwrap() = Some(cx.entity().entity_id());
        Self
    }
}

impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<TodoState>();
        let theme = cx.theme();

        let todos = state.todos.lock().unwrap();
        let search_query = state.search_query.lock().unwrap();
        let selected_category = state.selected_category.lock().unwrap();
        let compact_mode = *state.compact_mode.lock().unwrap();
        let editing_todo = *state.editing_todo.lock().unwrap();
        let new_todo_category = state.new_todo_category.lock().unwrap().clone();
        let edit_title = state.edit_title.lock().unwrap().clone();
        let edit_category = state.edit_category.lock().unwrap().clone();

        let filtered_todos: Vec<Todo> = todos
            .iter()
            .filter(|todo| {
                let matches_search = search_query.is_empty()
                    || todo.title.to_lowercase().contains(&search_query.to_lowercase());
                let matches_category = selected_category
                    .as_ref()
                    .map(|cat| &todo.category == cat)
                    .unwrap_or(true);
                matches_search && matches_category
            })
            .cloned()
            .collect();

        div()
            .size_full()
            .child(
                div()
                    .size_full()
                    .bg(theme.surface.base)
                    .p(px(24.))
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(components::todo_header::TodoHeader::render(compact_mode))
                    .child(components::todo_toolbar::TodoToolbar::render(&search_query, &selected_category))
                    .child(components::todo_form::TodoForm::render(new_todo_category))
                    .child(
                        div()
                            .flex_col()
                            .gap(px(12.))
                            .flex_grow()
                            .min_h_0()
                            .children(filtered_todos.into_iter().map(|todo| {
                                components::todo_item::TodoItem::render(&todo, compact_mode)
                            })),
                    ),
            )
            .when_some(editing_todo, |this, _| {
                this.child(components::todo_modal::TodoModal::render(edit_title, edit_category))
            })
    }
}
