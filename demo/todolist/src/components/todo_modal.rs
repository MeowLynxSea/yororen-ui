//! yororen-ui Modal Pattern
//!
//! This component demonstrates how to build modal dialogs in yororen-ui.
//!
//! ## Modal Pattern
//!
//! Modals in yororen-ui typically:
//! 1. Use an overlay to block interaction with underlying content
//! 2. Use the `modal` component for the dialog itself
//! 3. Support close via X button, cancel button, or overlay click
//! 4. Use content() and actions() sections
//!
//! ## Key Components Used
//!
//! - `modal` - The dialog container
//! - `modal().title()` - Dialog title
//! - `modal().content()` - Main dialog content
//! - `modal().actions()` - Action buttons (Save, Cancel, etc.)
//! - `modal().on_close()` - Handle close events
//! - `modal().closable()` - Enable close button
//!
//! ## State Management in Modals
//!
//! When dealing with modals, remember:
//! 1. Lock ordering matters - always release one lock before acquiring another
//! 2. Clear modal state (e.g., editing_todo = None) when closing

use gpui::prelude::FluentBuilder;
use gpui::{InteractiveElement, IntoElement, ParentElement, Styled, div, hsla, px};
use yororen_ui::component::{ComboBoxOption, button, combo_box, modal, text_input};
use yororen_ui::i18n::Translate;
use yororen_ui::theme::ActionVariantKind;

use crate::state::TodoState;
use crate::todo::TodoCategory;

/// Demonstrates yororen-ui modal dialog pattern
pub struct TodoModal;

impl TodoModal {
    /// Standard modal render pattern
    pub fn render(
        cx: &gpui::App,
        edit_title: String,
        edit_category: TodoCategory,
    ) -> impl IntoElement {
        let edit_title_key = cx.t("demo.todolist.edit_task");
        let task_title_key = cx.t("demo.todolist.task_title");
        let cancel_key = cx.t("common.cancel");
        let save_key = cx.t("common.save");

        let category_options: Vec<ComboBoxOption> = TodoCategory::all()
            .iter()
            .map(|c| ComboBoxOption::new(c.code(), cx.t(c.key())))
            .collect();

        let category_value = edit_category.code().to_string();

        // Outer container with overlay
        // The overlay prevents mouse events from reaching elements behind the modal
        div()
            .absolute()
            .inset_0()
            .flex()
            .justify_center()
            .items_center()
            // Semi-transparent background overlay
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .bg(hsla(0., 0., 0., 0.5)),
            )
            // Modal container
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        // yororen-ui modal component
                        modal()
                            .title(edit_title_key)
                            .width(px(400.))
                            .closable(true)
                            // Handle modal close (via X button)
                            .on_close(|_, cx| {
                                // Clear editing state when modal closes
                                let entity_id = {
                                    let state = cx.global::<TodoState>();
                                    *state.editing_todo.lock().unwrap() = None;
                                    state.notify_entity.lock().unwrap().clone()
                                };
                                if let Some(entity_id) = entity_id {
                                    cx.notify(entity_id);
                                }
                            })
                            // Modal content: title input and category dropdown
                            .content(
                                div()
                                    .flex_col()
                                    .gap(px(16.))
                                    .child(
                                        text_input("edit-title")
                                            .placeholder(task_title_key)
                                            .when(
                                                {
                                                    let state = cx.global::<TodoState>();
                                                    let mut needs_init =
                                                        state.edit_needs_init.lock().unwrap();
                                                    let do_init = *needs_init;
                                                    if do_init {
                                                        *needs_init = false;
                                                    }
                                                    do_init
                                                },
                                                |this| this.set_content(edit_title.clone()),
                                            )
                                            .on_change(|text, _window, cx| {
                                                let state = cx.global::<TodoState>();
                                                *state.edit_title.lock().unwrap() =
                                                    text.to_string();
                                            }),
                                    )
                                    .child(
                                        combo_box("edit-category")
                                            .value(&category_value)
                                            .options(category_options)
                                            .on_change(|value, _ev, _window, cx| {
                                                let state = cx.global::<TodoState>();
                                                if let Some(cat) = TodoCategory::all()
                                                    .into_iter()
                                                    .find(|c| c.code() == value)
                                                {
                                                    *state.edit_category.lock().unwrap() = cat;
                                                }
                                            }),
                                    ),
                            )
                            // Modal action buttons
                            .actions(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(8.))
                                    // Cancel button - closes modal without saving
                                    .child(button("cancel-edit").child(cancel_key).on_click(
                                        |_ev, _window, cx| {
                                            let entity_id = {
                                                let state = cx.global::<TodoState>();
                                                *state.editing_todo.lock().unwrap() = None;
                                                state.notify_entity.lock().unwrap().clone()
                                            };
                                            if let Some(entity_id) = entity_id {
                                                cx.notify(entity_id);
                                            }
                                        },
                                    ))
                                    // Save button - persists changes
                                    .child(
                                        button("save-edit")
                                            .variant(ActionVariantKind::Primary)
                                            .child(save_key)
                                            .on_click(|_ev, _window, cx| {
                                                // IMPORTANT: Lock ordering
                                                // First get the id and new values, release lock
                                                // This avoids deadlocks by not holding multiple locks
                                                let (id, title, category) = {
                                                    let state = cx.global::<TodoState>();
                                                    if let Some(id) =
                                                        *state.editing_todo.lock().unwrap()
                                                    {
                                                        let title = state
                                                            .edit_title
                                                            .lock()
                                                            .unwrap()
                                                            .clone();
                                                        let category = state
                                                            .edit_category
                                                            .lock()
                                                            .unwrap()
                                                            .clone();
                                                        (Some(id), title, category)
                                                    } else {
                                                        (None, String::new(), TodoCategory::Other)
                                                    }
                                                };

                                                // Then update the todo (after releasing first lock)
                                                if let Some(id) = id {
                                                    let entity_id = {
                                                        let state = cx.global::<TodoState>();
                                                        if let Some(todo) = state
                                                            .todos
                                                            .lock()
                                                            .unwrap()
                                                            .iter_mut()
                                                            .find(|t| t.id == id)
                                                        {
                                                            todo.title = title;
                                                            todo.category = category;
                                                        }
                                                        // Clear editing state
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
