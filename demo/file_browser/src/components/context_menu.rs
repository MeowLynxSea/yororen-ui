//! File Browser Context Menu Component
//!
//! Renders a context menu for file operations (copy/paste).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{AnyElement, InteractiveElement, IntoElement, ParentElement, Pixels, Styled, div, px};

use yororen_ui::component::{
    IconName, PopoverPlacement, button, divider, icon, label, popover,
};
use yororen_ui::theme::{ActionVariantKind, Theme};

use crate::clipboard::{ClipboardOp, FileClipboard};
use crate::fs_ops;
use crate::scan;
use crate::state::FileBrowserState;

/// Renders the context menu at the specified position
///
/// Provides Copy and Paste actions based on current selection and clipboard state.
pub fn render(
    theme: &Arc<Theme>,
    menu_position: Option<gpui::Point<Pixels>>,
    context_path: Option<PathBuf>,
    clipboard: Option<FileClipboard>,
) -> AnyElement {
    let can_copy = context_path.is_some();
    let can_paste = clipboard.is_some()
        && context_path
            .as_ref()
            .is_some_and(|p| p.is_dir() || p.parent().is_some());

    let menu = div()
        .py_1()
        .child(
            div()
                .px_3()
                .py_2()
                .text_color(theme.content.secondary)
                .child(label("Actions").inherit_color(true)),
        )
        .child(divider())
        .child(
            button("file-browser:menu:copy")
                .w_full()
                .px_3()
                .py_2()
                .rounded_md()
                .variant(ActionVariantKind::Neutral)
                .disabled(!can_copy)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(icon(IconName::Pencil).size(px(14.)).color(theme.content.primary))
                        .child("Copy"),
                )
                .on_click(move |_ev, window, cx| {
                    let entity_id = {
                        let state = cx.global::<FileBrowserState>();
                        let Some(path) = state.context_path.lock().unwrap().clone() else {
                            return;
                        };

                        *state.clipboard.lock().unwrap() = Some(FileClipboard {
                            op: ClipboardOp::Copy,
                            src: path,
                        });
                        *state.menu_open.lock().unwrap() = false;
                        *state.menu_position.lock().unwrap() = None;
                        *state.notify_entity.lock().unwrap()
                    };

                    if let Some(id) = entity_id {
                        cx.notify(id);
                    }
                    window.refresh();
                }),
        )
        .child(
            button("file-browser:menu:paste")
                .w_full()
                .px_3()
                .py_2()
                .rounded_md()
                .variant(ActionVariantKind::Neutral)
                .disabled(!can_paste)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(icon(IconName::Modpack).size(px(14.)).color(theme.content.primary))
                        .child("Paste"),
                )
                .on_click(move |_ev, window, cx| {
                    let (entity_id, result) = {
                        let state = cx.global::<FileBrowserState>();
                        let Some(clip) = state.clipboard.lock().unwrap().clone() else {
                            return;
                        };
                        let Some(target) = state.context_path.lock().unwrap().clone() else {
                            return;
                        };

                        let dst_dir = if target.is_dir() {
                            target
                        } else {
                            target.parent().unwrap_or(Path::new(".")).to_path_buf()
                        };

                        let file_name = clip
                            .src
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "copied".to_string());

                        let dst = fs_ops::unique_child_path(&dst_dir, &file_name);
                        let result = fs_ops::copy_path(&clip.src, &dst);

                        let root = state.root.lock().unwrap().clone();

                        *state.menu_open.lock().unwrap() = false;
                        *state.menu_position.lock().unwrap() = None;
                        ((root, *state.notify_entity.lock().unwrap()), result)
                    };

                    if let Ok(()) = result {
                        if let (root, Some(id)) = entity_id {
                            cx.notify(id);
                            scan::start_scan(root, window, cx);
                        }
                    }
                    window.refresh();
                }),
        );

    let (trigger_left, trigger_top) = menu_position
        .map(|p| (p.x, p.y))
        .unwrap_or((px(0.), px(0.)));

    div()
        .absolute()
        .inset_0()
        .occlude()
        .child(
            popover("file-browser:menu")
                .open(true)
                .placement(PopoverPlacement::BottomStart)
                .width(px(260.))
                .absolute()
                .left(trigger_left)
                .top(trigger_top)
                .on_close(move |window, cx| {
                    let entity_id = {
                        let state = cx.global::<FileBrowserState>();
                        *state.menu_open.lock().unwrap() = false;
                        *state.menu_position.lock().unwrap() = None;
                        *state.notify_entity.lock().unwrap()
                    };
                    if let Some(id) = entity_id {
                        cx.notify(id);
                    }
                    window.refresh();
                })
                .trigger(div().w(px(1.)).h(px(1.)))
                .content(menu),
        )
        .into_any_element()
}
