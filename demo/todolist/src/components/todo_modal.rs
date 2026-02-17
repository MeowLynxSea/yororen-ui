use gpui::{InteractiveElement, IntoElement, ParentElement, Styled, div, px, hsla};
use yororen_ui::component::{button, combo_box, modal, text_input, ComboBoxOption};
use yororen_ui::theme::ActionVariantKind;

use crate::state::TodoState;
use crate::todo::TodoCategory;

pub struct TodoModal;

impl TodoModal {
    pub fn render(edit_title: String, edit_category: TodoCategory) -> impl IntoElement {
        let category_options: Vec<ComboBoxOption> = TodoCategory::all()
            .iter()
            .map(|c| ComboBoxOption::new(c.label(), c.label()))
            .collect();

        let category_label = edit_category.label().to_string();

        // 使用 overlay 容器来阻止鼠标事件穿透
        div()
            .absolute()
            .inset_0()
            .flex()
            .justify_center()
            .items_center()
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .bg(hsla(0., 0., 0., 0.5)),
            )
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        modal()
                            .title("编辑任务")
                            .width(px(400.))
                            .closable(true)
                            .on_close(|_, cx| {
                                let entity_id = {
                                    let state = cx.global::<TodoState>();
                                    *state.editing_todo.lock().unwrap() = None;
                                    state.notify_entity.lock().unwrap().clone()
                                };
                                if let Some(entity_id) = entity_id {
                                    cx.notify(entity_id);
                                }
                            })
                            .content(
                                div()
                                    .flex_col()
                                    .gap(px(16.))
                                    .child(
                                        text_input("edit-title")
                                            .placeholder("任务标题")
                                            .set_content(edit_title)
                                            .on_change(|text, _window, cx| {
                                                let state = cx.global::<TodoState>();
                                                *state.edit_title.lock().unwrap() = text.to_string();
                                            }),
                                    )
                                    .child(
                                        combo_box("edit-category")
                                            .value(&category_label)
                                            .options(category_options)
                                            .on_change(|value, _ev, _window, cx| {
                                                let state = cx.global::<TodoState>();
                                                if let Some(cat) = TodoCategory::all().into_iter().find(|c| c.label() == value) {
                                                    *state.edit_category.lock().unwrap() = cat;
                                                }
                                            }),
                                    ),
                            )
                            .actions(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(8.))
                                    .child(
                                        button("cancel-edit")
                                            .child("取消")
                                            .on_click(|_ev, _window, cx| {
                                                let entity_id = {
                                                    let state = cx.global::<TodoState>();
                                                    *state.editing_todo.lock().unwrap() = None;
                                                    state.notify_entity.lock().unwrap().clone()
                                                };
                                                if let Some(entity_id) = entity_id {
                                                    cx.notify(entity_id);
                                                }
                                            }),
                                    )
                                    .child(
                                        button("save-edit")
                                            .variant(ActionVariantKind::Primary)
                                            .child("保存")
                                            .on_click(|_ev, _window, cx| {
                                                // 先获取 id 和新值，释放锁
                                                let (id, title, category) = {
                                                    let state = cx.global::<TodoState>();
                                                    if let Some(id) = *state.editing_todo.lock().unwrap() {
                                                        let title = state.edit_title.lock().unwrap().clone();
                                                        let category = state.edit_category.lock().unwrap().clone();
                                                        (Some(id), title, category)
                                                    } else {
                                                        (None, String::new(), TodoCategory::Other)
                                                    }
                                                };

                                                // 然后更新 todo
                                                if let Some(id) = id {
                                                    let entity_id = {
                                                        let state = cx.global::<TodoState>();
                                                        if let Some(todo) = state.todos.lock().unwrap().iter_mut().find(|t| t.id == id) {
                                                            todo.title = title;
                                                            todo.category = category;
                                                        }
                                                        *state.editing_todo.lock().unwrap() = None;
                                                        state.notify_entity.lock().unwrap().clone()
                                                    };
                                                    if let Some(entity_id) = entity_id {
                                                        cx.notify(entity_id);
                                                    }
                                                }
                                            }),
                                    ),
                            ),
                    ),
            )
    }
}
