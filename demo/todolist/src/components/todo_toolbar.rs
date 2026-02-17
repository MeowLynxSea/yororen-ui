use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{combo_box, search_input, ComboBoxOption};

use crate::state::TodoState;
use crate::todo::TodoCategory;

pub struct TodoToolbar;

impl TodoToolbar {
    pub fn render(search_query: &str, selected_category: &Option<TodoCategory>) -> impl IntoElement {
        let category_options: Vec<ComboBoxOption> = TodoCategory::all()
            .iter()
            .map(|c| ComboBoxOption::new(c.label(), c.label()))
            .collect();

        let mut search_options = vec![ComboBoxOption::new("all", "全部分类")];
        search_options.extend(category_options.clone());

        let selected_value = selected_category
            .as_ref()
            .map(|c| c.label().to_string())
            .unwrap_or_else(|| "全部分类".to_string());

        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .child(
                search_input("search")
                    .w(px(200.))
                    .placeholder("搜索任务...")
                    .on_change(|text, _window, cx| {
                        let state = cx.global::<TodoState>();
                        *state.search_query.lock().unwrap() = text.to_string();
                    }),
            )
            .child(
                combo_box("category-filter")
                    .placeholder("全部分类")
                    .value(&selected_value)
                    .options(search_options)
                    .on_change(|value, _ev, _window, cx| {
                        let state = cx.global::<TodoState>();
                        let category = if value == "all" {
                            None
                        } else {
                            TodoCategory::all().into_iter().find(|c| c.label() == value)
                        };
                        *state.selected_category.lock().unwrap() = category;
                    }),
            )
    }
}
