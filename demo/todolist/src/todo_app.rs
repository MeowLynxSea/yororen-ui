//! yororen-ui Root Component Pattern
//!
//! This file demonstrates the **standard pattern** for building the root component
//! in any yororen-ui application.
//!
//! ## Root Component Responsibilities
//!
//! A root component (the one passed to `cx.open_window`) typically:
//! 1. Implements the `Render` trait from gpui
//! 2. Reads global state via `cx.global::<T>()`
//! 3. Derives UI state from global state (filtering, sorting, etc.)
//! 4. Renders child components
//! 5. Handles global notifications
//!
//! ## Render Trait Pattern
//!
//! ```
//! impl Render for MyApp {
//!     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         // Read global state
//!         let state = cx.global::<MyState>();
//!         // Derive UI state
//!         let filtered_data = state.items.lock().unwrap().filter(...);
//!         // Build UI
//!         div().children(...)
//!     }
//! }
//! ```
//!
//! ## Using This Pattern
//!
//! Copy this structure for your yororen-ui app's root component.

use gpui::{
    prelude::FluentBuilder,
    Context, IntoElement, ParentElement,
    Render, Styled, Window, div, px,
};
use yororen_ui::theme::ActiveTheme;

use crate::components;
use crate::todo::Todo;
use crate::state::TodoState;

/// Root component - the entry point for your application's UI tree
///
/// This is the component passed to `cx.open_window()` in main().
/// It serves as the parent for all other components in the application.
pub struct TodoApp;

impl TodoApp {
    /// Initializes the root component
    ///
    /// IMPORTANT: Store your entity_id in global state for notification purposes.
    /// This allows other components to trigger re-renders of this component.
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Store our entity_id so other components can notify us of changes
        let state = cx.global::<TodoState>();
        *state.notify_entity.lock().unwrap() = Some(cx.entity().entity_id());
        Self
    }
}

/// Render trait - the core of gpui component system
///
/// This is called by gpui when:
/// - The component is first displayed
/// - `cx.notify(entity_id)` is called
/// - Global state changes that this component depends on
impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app: &gpui::App = &*cx;

        // Step 1: Read global state
        let state = cx.global::<TodoState>();
        let theme = cx.theme();

        // Step 2: Lock and read state fields
        let todos = state.todos.lock().unwrap();
        let search_query = state.search_query.lock().unwrap();
        let selected_category = state.selected_category.lock().unwrap();
        let compact_mode = *state.compact_mode.lock().unwrap();
        let editing_todo = *state.editing_todo.lock().unwrap();
        let new_todo_category = state.new_todo_category.lock().unwrap().clone();
        let edit_title = state.edit_title.lock().unwrap().clone();
        let edit_category = state.edit_category.lock().unwrap().clone();

        // Step 3: Derive UI state (filtering, sorting, etc.)
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

        // Step 4: Build UI tree using fluent builder pattern
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
                    // Step 5: Render child components
                    .child(components::todo_header::TodoHeader::render(app, compact_mode))
                    .child(components::todo_toolbar::TodoToolbar::render(
                        app,
                        &search_query,
                        &selected_category,
                    ))
                    .child(components::todo_form::TodoForm::render(app, new_todo_category))
                    .child(
                        div()
                            .flex_col()
                            .gap(px(12.))
                            .flex_grow()
                            .min_h_0()
                            .children(filtered_todos.into_iter().map(|todo| {
                                components::todo_item::TodoItem::render(app, &todo, compact_mode)
                            })),
                    ),
            )
            // Conditional rendering: show modal when editing
            .when_some(editing_todo, |this, _| {
                this.child(components::todo_modal::TodoModal::render(app, edit_title, edit_category))
            })
    }
}
