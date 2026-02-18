use std::path::PathBuf;
use std::sync::Arc;

use gpui::{AnyElement, IntoElement, ParentElement, Styled, Window, div, px};

use yororen_ui::component::{
    SelectionMode, TreeNode, TreeState, button, context_menu_trigger, empty_state, label, tree,
};
use yororen_ui::theme::{ActionVariantKind, Theme};
use yororen_ui::widget::virtual_list_state;

use crate::actions;
use crate::state::FileBrowserState;

pub struct FileBrowserTreePanel;

impl FileBrowserTreePanel {
    pub fn render(
        theme: &Arc<Theme>,
        root: PathBuf,
        tree_nodes: Vec<TreeNode>,
        is_scanning: bool,
    ) -> AnyElement {
        let is_empty = tree_nodes.is_empty();

        let empty = empty_state("file-browser:empty")
            .title("Nothing to show")
            .description("This folder is empty or cannot be read.")
            .action(
                button("file-browser:empty:pick-root")
                    .variant(ActionVariantKind::Primary)
                    .child("Pick another root")
                    .on_click(move |_ev, window: &mut Window, cx| {
                        actions::prompt_for_root(window, cx);
                    }),
            );

        let tree_view: AnyElement = if is_empty {
            div()
                .flex()
                .items_center()
                .justify_center()
                .flex_grow()
                .child(if is_scanning {
                    label("Scanning...").muted(true).into_any_element()
                } else {
                    empty.into_any_element()
                })
                .into_any_element()
        } else {
            let list_state =
                virtual_list_state(tree_nodes.len(), gpui::ListAlignment::Top, px(32.));
            tree(TreeState::new(), &tree_nodes)
                .id("file-browser:tree")
                .virtualized(true)
                .list_state(list_state)
                .selection_mode(SelectionMode::Single)
                .on_item_click(|id, _ev, _window, cx| {
                    let path = PathBuf::from(id.to_string());
                    let state = cx.global::<FileBrowserState>();
                    *state.selected_path.lock().unwrap() = Some(path);

                    let id = *state.notify_entity.lock().unwrap();
                    if let Some(id) = id {
                        cx.notify(id);
                    }
                })
                .on_item_context_menu(|id, ev, _window, cx| {
                    let path = PathBuf::from(id.to_string());
                    let state = cx.global::<FileBrowserState>();

                    *state.context_path.lock().unwrap() = Some(path);
                    *state.menu_position.lock().unwrap() = Some(ev.position);
                    *state.menu_open.lock().unwrap() = true;

                    let notify_id = *state.notify_entity.lock().unwrap();
                    if let Some(notify_id) = notify_id {
                        cx.notify(notify_id);
                    }
                })
                .into_any_element()
        };

        context_menu_trigger("file-browser:context")
            .consume(false)
            .flex()
            .flex_col()
            .flex_grow()
            .min_h_0()
            .rounded_lg()
            .bg(theme.surface.raised)
            .border_1()
            .border_color(theme.border.divider)
            .p_2()
            .on_open(move |ev, _window, cx| {
                let state = cx.global::<FileBrowserState>();

                let selected = state.selected_path.lock().unwrap().clone();
                *state.context_path.lock().unwrap() = Some(selected.unwrap_or_else(|| root.clone()));
                *state.menu_position.lock().unwrap() = Some(ev.position);
                *state.menu_open.lock().unwrap() = true;

                let id = *state.notify_entity.lock().unwrap();
                if let Some(id) = id {
                    cx.notify(id);
                }
            })
            .child(tree_view)
            .into_any_element()
    }
}
