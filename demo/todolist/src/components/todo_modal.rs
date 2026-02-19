//! yororen-ui Modal Dialog Component
//!
//! This module demonstrates the standard pattern for building modal dialogs in yororen-ui applications.
//! The modal is used for editing existing todo items, providing a focused interface for modifying task details.
//!
//! ## Modal Dialog Architecture
//!
//! A modal dialog in yororen-ui typically consists of several key elements:
//!
//! 1. **Overlay Layer**: A semi-transparent background that covers the entire viewport, preventing
//!    interaction with underlying content and drawing attention to the modal
//! 2. **Modal Container**: The actual dialog box that contains the content and actions
//! 3. **Title Bar**: Displays the dialog's purpose (e.g., "Edit Task")
//! 4. **Content Area**: Contains form inputs for user interaction (title, category, etc.)
//! 5. **Actions Area**: Contains action buttons (Save, Cancel) for form submission
//!
//! ## Modal State Management
//!
//! When implementing modals in yororen-ui, proper state management is critical:
//!
//! - **Opening the Modal**: Set `editing_todo` to the ID of the item being edited, and populate
//!   the edit buffers (`edit_title`, `edit_category`) with the current values
//! - **Closing the Modal**: Always clear the `editing_todo` state (set to `None`) when the modal
//!   closes, regardless of whether the user saved or cancelled
//! - **Lock Ordering**: When accessing multiple mutex-protected state fields, always acquire and
//!   release locks in a consistent order to avoid deadlocks. The recommended pattern is to read
//!   all needed values under a single lock, then release it before acquiring any other locks
//!
//! ## Key Components Used
//!
//! - `modal` - The core modal dialog container component
//! - `modal().title()` - Sets the dialog title displayed in the title bar
//! - `modal().content()` - Contains the main form content
//! - `modal().actions()` - Contains the action buttons (Save, Cancel)
//! - `modal().on_close()` - Callback fired when the modal is closed (via X button, Cancel, or overlay click)
//! - `modal().closable()` - Enables the close button in the title bar
//! - `text_input` - Single-line text input for task title
//! - `combo_box` - Dropdown selection for task category
//! - `button` - Action buttons with primary/secondary variants

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
