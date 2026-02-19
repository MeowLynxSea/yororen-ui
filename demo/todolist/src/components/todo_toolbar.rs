//! yororen-ui Toolbar/Search Pattern
//!
//! Demonstrates search and filter patterns in yororen-ui.
//!
//! ## Key Components Used
//!
//! - `search_input` - Text input for search
//! - `combo_box` - Dropdown for category selection
//! - `ComboBoxOption` - Options for dropdowns

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
                        let state = cx.global::<TodoState>();
                        *state.search_query.lock().unwrap() = text.to_string();
                    }),
            )
            // Category filter dropdown
            .child(
                combo_box("category-filter")
                    .placeholder(all_categories_label)
                    .value(&selected_value)
                    .options(search_options)
                    .on_change(|value, _ev, _window, cx| {
                        let state = cx.global::<TodoState>();
                        let category = if value == "all" {
                            None
                        } else {
                            TodoCategory::all().into_iter().find(|c| c.code() == value)
                        };
                        *state.selected_category.lock().unwrap() = category;
                    }),
            )
    }
}
