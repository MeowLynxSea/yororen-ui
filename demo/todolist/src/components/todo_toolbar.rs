//! yororen-ui Toolbar Component
//!
//! This module demonstrates the standard pattern for building search and filter toolbars in yororen-ui applications.
//! The TodoToolbar component provides controls for searching and filtering the todo list.
//!
//! ## Toolbar Design Pattern
//!
//! Toolbars in yororen-ui applications typically contain controls the for manipulating view or data.
//! This component demonstrates a common pattern combining:
//!
//! - **Search Input**: A text field for filtering items by their text content
//! - **Category Filter**: A dropdown for filtering items by category
//!
//! ## Filter State Management
//!
//! Both search and filter controls manage their state in the global state store:
//!
//! - `search_query`: Stores the current search text, used to filter todos by title
//! - `selected_category`: Stores the currently selected category filter (None means "all categories")
//!
//! When these values change, the root component re-renders and applies the filters to derive
//! the visible list of todos.
//!
//! ## ComboBox Options Pattern
//!
//! The category filter demonstrates a common pattern for dropdown options:
//!
//! 1. Create a vector of `ComboBoxOption` items from the domain enum using `.iter()` and `.map()`
//! 2. Add an "All" option at the beginning to allow showing all items
//! 3. Map the current selection to the option value for display
//!
//! ## Key Components Used
//!
//! - `search_input` - Specialized text input with search icon and placeholder for search queries
//! - `combo_box` - Dropdown selection component for category filtering
//! - `ComboBoxOption` - Represents individual options in the dropdown, with a code value and display label

use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{combo_box, search_input, ComboBoxOption};
use yororen_ui::i18n::Translate;

use crate::state::TodoState;
use crate::todo::TodoCategory;

/// Toolbar with search and filter controls
pub struct TodoToolbar;
impl TodoToolbar {
    /// Standard toolbar pattern with search and filter
    pub fn render(
        cx: &gpui::App,
        _search_query: &str,
        selected_category: &Option<TodoCategory>,
    ) -> impl IntoElement {
        let search_placeholder = cx.t("demo.todolist.search_placeholder");
        let all_categories_label = cx.t("demo.todolist.all_categories");

        // Build category options
        let category_options: Vec<ComboBoxOption> = TodoCategory::all()
            .iter()
            .map(|c| ComboBoxOption::new(c.code(), cx.t(c.key())))
            .collect();

        // Add "All" option
        let mut search_options = vec![ComboBoxOption::new("all", all_categories_label.clone())];
        search_options.extend(category_options.clone());

        let selected_value = selected_category
            .as_ref()
            .map(|c| c.code().to_string())
            .unwrap_or_else(|| "all".to_string());

        div()
            .flex()
            .items_center()
            .gap(px(12.))
            // Search input - on_change fires on every keystroke
            .child(
                search_input("search")
                    .w(px(200.))
                    .placeholder(search_placeholder)
                    .on_change(|text, _window, cx| {
                        let model = cx.global::<TodoState>().model.clone();
                        model.update(cx, |model, cx| {
                            model.search_query = text.to_string();
                            cx.notify();
                        });
                    }),
            )
            // Category filter dropdown
            .child(
                combo_box("category-filter")
                    .placeholder(all_categories_label)
                    .value(&selected_value)
                    .options(search_options)
                    .on_change(|value, _ev, _window, cx| {
                        let category = if value == "all" {
                            None
                        } else {
                            TodoCategory::all().into_iter().find(|c| c.code() == value)
                        };
                        let model = cx.global::<TodoState>().model.clone();
                        model.update(cx, |model, cx| {
                            model.selected_category = category;
                            cx.notify();
                        });
                    }),
            )
    }
}
