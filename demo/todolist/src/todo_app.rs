//! yororen-ui Root Application Component
//!
//! This module demonstrates the standard pattern for building the root component in yororen-ui applications.
//! The root component serves as the top-level container for all other UI elements in the application.
//!
//! ## Root Component Responsibilities
//!
//! The root component (the one passed to `cx.open_window()`) is responsible for:
//!
//! 1. **Implementing the Render Trait**: The `Render` trait from gpui is the core of the component system.
//!    It defines how the component transforms its state into UI elements.
//! 2. **Reading Global State**: Access application-wide state via `cx.global::<T>()` to retrieve
//!    shared data that persists across the application lifecycle.
//! 3. **Deriving UI State**: Transform raw state data into presentation-ready data by applying
//!    filters, sorting, or other transformations needed for rendering.
//! 4. **Composing Child Components**: Assemble the UI by combining child components, passing them
//!    the data and callbacks they need to function correctly.
//! 5. **Handling Notifications**: Respond to `cx.notify()` calls that signal state changes requiring re-renders.
//!
//! ## Render Trait Implementation Pattern
//!
//! ```ignore
//! impl Render for MyApp {
//!     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         // Step 1: Read global state
//!         let state = cx.global::<MyState>();
//!
//!         // Step 2: Derive UI state (filtering, sorting, etc.)
//!         let filtered_data = state.items.lock().unwrap().filter(...);
//!
//!         // Step 3: Build UI using fluent builder pattern
//!         div().children(...)
//!     }
//! }
//! ```
//!
//! ## Notification System
//!
//! The root component plays a crucial role in the notification system:
//! - Other components need to know the root component's entity ID to trigger re-renders
//! - Store the entity ID in global state during initialization (`Self::new()`)
//! - Components call `cx.notify(entity_id)` after modifying state to trigger a re-render
//!
//! ## Conditional Rendering
//!
//! The root component often handles conditional rendering of overlays like modals:
//! - Check if a modal should be displayed (e.g., `editing_todo.is_some()`)
//! - Use `.when_some()` or `.when()` to conditionally render elements
//! - Pass the necessary state to child components for rendering

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
    pub fn new(_cx: &mut Context<Self>) -> Self {
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

        // Step 2: Read model fields (no locks)
        let model = state.model.read(app);
        let compact_mode = model.compact_mode;
        let editing_todo = model.editing_todo;
        let new_todo_category = model.new_todo_category.clone();
        let edit_title = model.edit_title.clone();
        let edit_category = model.edit_category.clone();
        let search_query = model.search_query.clone();
        let selected_category = model.selected_category.clone();
        let todos = model.todos.clone();

        // Step 3: Derive UI state (filtering, sorting, etc.)
        let filtered_todos: Vec<Todo> = todos
            .into_iter()
            .filter(|todo| {
                let matches_search = search_query.is_empty()
                    || todo.title.to_lowercase().contains(&search_query.to_lowercase());
                let matches_category = selected_category
                    .as_ref()
                    .map(|cat| &todo.category == cat)
                    .unwrap_or(true);
                matches_search && matches_category
            })
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
