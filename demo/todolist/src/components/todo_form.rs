use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{button, combo_box, text_input, ComboBoxOption};
use yororen_ui::theme::ActionVariantKind;

use crate::state::TodoState;
use crate::todo::{Todo, TodoCategory};

pub struct TodoForm;

impl TodoForm {
    pub fn render(current_category: TodoCategory) -> impl IntoElement {
        let category_options: Vec<ComboBoxOption> = TodoCategory::all()
            .iter()
            .map(|c| ComboBoxOption::new(c.label(), c.label()))
            .collect();

        let category_label = current_category.label().to_string();

        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .child(
                text_input("new-todo")
                    .gap_2()
                    .placeholder("添加新任务...")
                    .on_change(|text, _window, cx| {
                        let state = cx.global::<TodoState>();
                        *state.new_todo_title.lock().unwrap() = text.to_string();
                    }),
            )
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
            .child(
                button("add-btn")
                    .gap_2()
                    .variant(ActionVariantKind::Primary)
                    .child("添加")
                    .on_click(|_ev, _window, cx| {
                        let state = cx.global::<TodoState>();
                        let title = state.new_todo_title.lock().unwrap().clone();
                        if !title.trim().is_empty() {
                            let category = state.new_todo_category.lock().unwrap().clone();
                            let todo = Todo::new(title.trim().to_string(), category);
                            state.todos.lock().unwrap().insert(0, todo);
                            *state.new_todo_title.lock().unwrap() = String::new();
                            let entity_id = state.notify_entity.lock().unwrap().clone();
                            if let Some(id) = entity_id {
                                cx.notify(id);
                            }
                        }
                    }),
            )
    }
}
