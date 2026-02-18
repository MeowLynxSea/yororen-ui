use gpui::{AnyElement, Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use yororen_ui::component::divider;
use yororen_ui::theme::ActiveTheme;

use crate::components;
use crate::scan;
use crate::state::FileBrowserState;

pub struct FileBrowserApp;

impl FileBrowserApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.global::<FileBrowserState>();
        *state.notify_entity.lock().unwrap() = Some(cx.entity().entity_id());
        Self
    }
}

impl Render for FileBrowserApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<FileBrowserState>();
        let theme = cx.theme().clone();

        let root = state.root.lock().unwrap().clone();
        let selected_path = state.selected_path.lock().unwrap().clone();
        let context_path = state.context_path.lock().unwrap().clone();
        let clipboard = state.clipboard.lock().unwrap().clone();
        let menu_open = *state.menu_open.lock().unwrap();
        let menu_position = state.menu_position.lock().unwrap().clone();

        let tree_nodes = state.tree_nodes.lock().unwrap().clone();
        let is_scanning = *state.is_scanning.lock().unwrap();

        if tree_nodes.is_empty() && !is_scanning {
            let root = root.clone();
            window
                .spawn(cx, async move |cx| {
                    let _ = cx.update(|window, cx| scan::start_scan(root, window, cx));
                })
                .detach();
        }

        let header = components::header::FileBrowserHeader::render(&root);
        let details = components::details::FileBrowserDetails::render(&selected_path, &clipboard);

        let tree_panel = components::tree_panel::FileBrowserTreePanel::render(
            &theme,
            root.clone(),
            tree_nodes,
            is_scanning,
        );

        let context_menu: Option<AnyElement> = if menu_open {
            Some(components::context_menu::render(
                &theme,
                menu_position,
                context_path,
                clipboard.clone(),
            ))
        } else {
            None
        };

        let mut root_view = div().size_full().bg(theme.surface.base).relative();
        root_view = root_view.child(
            div()
                .size_full()
                .p(px(20.))
                .flex()
                .flex_col()
                .min_h_0()
                .gap(px(12.))
                .child(header)
                .child(details)
                .child(divider())
                .child(tree_panel),
        );

        if let Some(context_menu) = context_menu {
            root_view = root_view.child(context_menu);
        }

        root_view
    }
}
