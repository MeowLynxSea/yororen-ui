use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::{AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Pixels, Render, Styled, Window, div, px};

use yororen_ui::component::{
    ArcTreeNode, IconName, PopoverPlacement, SelectionMode, TreeNode, TreeState,
    button, context_menu_trigger, divider, empty_state, icon, label, popover, tree,
};
use yororen_ui::widget::virtual_list_state;
use yororen_ui::theme::ActionVariantKind;
use yororen_ui::theme::ActiveTheme;

use crate::state::{ClipboardOp, FileBrowserState, FileClipboard};

fn notify_file_browser(cx: &mut gpui::App) {
    let id = {
        let state = cx.global::<FileBrowserState>();
        *state.notify_entity.lock().unwrap()
    };
    if let Some(id) = id {
        cx.notify(id);
    }
}

fn set_children_by_id(nodes: &mut [TreeNode], parent_id: &str, children: Vec<TreeNode>) -> bool {
    for node in nodes {
        if node.id.to_string() == parent_id {
            node.children = children;
            node.has_children = !node.children.is_empty();
            return true;
        }
        if set_children_by_id(&mut node.children, parent_id, children.clone()) {
            return true;
        }
    }
    false
}

fn read_dir_nodes(dir: &Path) -> Vec<(TreeNode, Option<PathBuf>)> {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| {
        let ty = e.file_type().ok();
        let is_dir = ty.map(|t| t.is_dir()).unwrap_or(false);
        (if is_dir { 0 } else { 1 }, e.file_name())
    });

    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        let is_dir = entry
            .file_type()
            .ok()
            .map(|t| t.is_dir())
            .unwrap_or(false);

        let mut data = ArcTreeNode::new(file_name);
        data.icon = Some(if is_dir {
            "icons/server.svg".to_string()
        } else {
            "icons/user.svg".to_string()
        });

        let id = path.to_string_lossy().to_string();
        let node = TreeNode {
            id: id.into(),
            data,
            children: Vec::new(),
            expanded: false,
            selected: false,
            checked: yororen_ui::component::TreeCheckedState::Unchecked,
            depth: 0,
            has_children: is_dir,
        };
        out.push((node, is_dir.then_some(path)));
    }

    out
}

fn start_scan(root: PathBuf, window: &mut Window, cx: &mut gpui::App) {
    let state = cx.global::<FileBrowserState>();
    let generation = {
        let mut generation_lock = state.scan_generation.lock().unwrap();
        *generation_lock = generation_lock.wrapping_add(1);
        *generation_lock
    };

    *state.is_scanning.lock().unwrap() = true;
    state.tree_nodes.lock().unwrap().clear();
    notify_file_browser(cx);

    window
        .spawn(cx, async move |cx| {
            let max_depth = 3usize;
            let mut stack: Vec<(PathBuf, usize)> = vec![(root.clone(), 0)];

            while let Some((dir, depth)) = stack.pop() {
                if depth > max_depth {
                    continue;
                }

                let dir_for_bg = dir.clone();
                let scanned = cx
                    .background_executor()
                    .await_on_background(async move { read_dir_nodes(&dir_for_bg) })
                    .await;

                let mut children: Vec<TreeNode> = Vec::with_capacity(scanned.len());
                let mut next_dirs: Vec<PathBuf> = Vec::new();
                for (node, child_dir) in scanned {
                    if let Some(child_dir) = child_dir {
                        next_dirs.push(child_dir);
                    }
                    children.push(node);
                }

                let dir_id = dir.to_string_lossy().to_string();
                let _ = cx.update(|_window, cx| {
                    let state = cx.global::<FileBrowserState>();
                    if *state.scan_generation.lock().unwrap() != generation {
                        return;
                    }

                    let mut nodes = state.tree_nodes.lock().unwrap();
                    if depth == 0 {
                        *nodes = children;
                    } else {
                        let _ = set_children_by_id(&mut nodes, &dir_id, children);
                    }
                });

                let _ = cx.update(|_window, cx| notify_file_browser(cx));

                // Yield between directories so scrolling remains responsive.
                cx.background_executor().timer(Duration::from_millis(8)).await;

                for child_dir in next_dirs {
                    stack.push((child_dir, depth + 1));
                }
            }

            let _ = cx.update(|_window, cx| {
                let state = cx.global::<FileBrowserState>();
                if *state.scan_generation.lock().unwrap() != generation {
                    return;
                }

                *state.is_scanning.lock().unwrap() = false;
                notify_file_browser(cx);
            });
        })
        .detach();
}

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
        let is_empty = tree_nodes.is_empty();

        if is_empty && !is_scanning {
            let root = root.clone();
            window
                .spawn(cx, async move |cx| {
                    let _ = cx.update(|window, cx| start_scan(root, window, cx));
                })
                .detach();
        }

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(label("File Browser Demo").strong(true))
                    .child(
                        label(root.to_string_lossy().to_string())
                            .muted(true)
                            .mono(true)
                            .ellipsis(true),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        button("file-browser:refresh")
                            .variant(ActionVariantKind::Neutral)
                            .child("Refresh")
                            .on_click(move |_ev, window, cx| {
                                let root = {
                                    let state = cx.global::<FileBrowserState>();
                                    state.root.lock().unwrap().clone()
                                };

                                start_scan(root, window, cx);
                            }),
                    )
                    .child(
                        button("file-browser:root")
                            .variant(ActionVariantKind::Neutral)
                            .child("Pick Root")
                            .on_click(move |_ev, window, cx| {
                                let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
                                    files: false,
                                    directories: true,
                                    multiple: false,
                                    prompt: Some("Select root directory".into()),
                                });

                                window
                                    .spawn(cx, async move |cx| {
                                        let result = receiver.await;
                                        cx.update(|window, cx| {
                                            let selected = match result {
                                                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                                                _ => None,
                                            };

                                            if let Some(path) = selected {
                                                let state = cx.global::<FileBrowserState>();
                                                *state.root.lock().unwrap() = path;
                                                *state.selected_path.lock().unwrap() = None;
                                                *state.context_path.lock().unwrap() = None;
                                                *state.menu_open.lock().unwrap() = false;
                                                *state.menu_position.lock().unwrap() = None;
                                                let root = state.root.lock().unwrap().clone();
                                                let entity_id = *state.notify_entity.lock().unwrap();
                                                let _ = state;
                                                let _ = entity_id;
                                                start_scan(root, window, cx);
                                            }
                                        })
                                        .ok();
                                    })
                                    .detach();
                            }),
                    ),
            );

        let details = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(label("Selected:").muted(true))
            .child(label(path_or_dash(&selected_path)).mono(true).ellipsis(true))
            .child(div().w(px(24.)))
            .child(label("Clipboard:").muted(true))
            .child(label(clipboard_label(&clipboard)).mono(true).ellipsis(true));

        let empty = empty_state("file-browser:empty")
            .title("Nothing to show")
            .description("This folder is empty or cannot be read.")
            .action(
                button("file-browser:empty:pick-root")
                    .variant(ActionVariantKind::Primary)
                    .child("Pick another root")
                    .on_click(move |_ev, window, cx| {
                        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
                            files: false,
                            directories: true,
                            multiple: false,
                            prompt: Some("Select root directory".into()),
                        });

                        window
                            .spawn(cx, async move |cx| {
                                let result = receiver.await;
                                cx.update(|window, cx| {
                                    let selected = match result {
                                        Ok(Ok(Some(paths))) => paths.into_iter().next(),
                                        _ => None,
                                    };

                                    if let Some(path) = selected {
                                        let state = cx.global::<FileBrowserState>();
                                        *state.root.lock().unwrap() = path;
                                        *state.selected_path.lock().unwrap() = None;
                                        *state.context_path.lock().unwrap() = None;
                                        *state.menu_open.lock().unwrap() = false;
                                        *state.menu_position.lock().unwrap() = None;
                                        let root = state.root.lock().unwrap().clone();
                                        let entity_id = *state.notify_entity.lock().unwrap();
                                        let _ = state;
                                        let _ = entity_id;
                                        start_scan(root, window, cx);
                                    }
                                })
                                .ok();
                            })
                            .detach();
                    }),
            );

        let context_menu: Option<AnyElement> = menu_open
            .then(|| build_context_menu(window, cx, menu_position, &context_path, &clipboard))
            .map(IntoElement::into_any_element);

        let tree_view: gpui::AnyElement = if is_empty {
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
            // Note: Tree uses internal keyed state; give it a stable id.
            // Enable virtualization for scrolling support.
            let list_state = virtual_list_state(tree_nodes.len(), gpui::ListAlignment::Top, px(32.));
            tree(TreeState::new(), &tree_nodes)
                .id("file-browser:tree")
                .virtualized(true)
                .list_state(list_state)
                .selection_mode(SelectionMode::Single)
                .on_item_click(|id, _ev, _window, cx| {
                    // Tree node IDs are encoded paths in this demo.
                    let path = PathBuf::from(id.to_string());
                    let (entity_id, _did_set) = {
                        let state = cx.global::<FileBrowserState>();
                        *state.selected_path.lock().unwrap() = Some(path);
                        (*state.notify_entity.lock().unwrap(), ())
                    };
                    let _ = entity_id;
                })
                .into_any_element()
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
                .child(
                    // Wrap the tree in a context menu trigger so right-click always has a surface.
                    // Actual context path is set by per-row triggers in custom rows (see below).
                    context_menu_trigger("file-browser:context")
                        // Don't consume events on this wrapper; otherwise the scroll wheel
                        // can be intercepted before it reaches the virtualized list.
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
                            // Fallback behavior: if we didn't right-click a specific row,
                            // open the menu for the root directory.
                            *state.context_path.lock().unwrap() = Some(root.clone());
                            *state.menu_position.lock().unwrap() = Some(ev.position);
                            *state.menu_open.lock().unwrap() = true;
                            let entity_id = *state.notify_entity.lock().unwrap();
                            let _ = state;
                            if let Some(id) = entity_id {
                                cx.notify(id);
                            }
                        })
                        .child(tree_view),
                ),
        );

        if let Some(context_menu) = context_menu {
            root_view = root_view.child(context_menu);
        }

        root_view
    }
}

fn path_or_dash(path: &Option<PathBuf>) -> String {
    path.as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn clipboard_label(clipboard: &Option<FileClipboard>) -> String {
    let Some(clipboard) = clipboard else {
        return "-".to_string();
    };

    match clipboard.op {
        ClipboardOp::Copy => format!("copy: {}", clipboard.src.to_string_lossy()),
    }
}

fn build_context_menu(
    _window: &mut Window,
    cx: &mut Context<FileBrowserApp>,
    menu_position: Option<gpui::Point<Pixels>>,
    context_path: &Option<PathBuf>,
    clipboard: &Option<FileClipboard>,
) -> impl IntoElement {
    let theme = cx.theme().clone();

    let can_copy = context_path.is_some();
    let can_paste = context_path
        .as_ref()
        .and_then(|p| p.parent().map(|_| p.clone()))
        .is_some()
        && clipboard.is_some();

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
                        .child("Paste")
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

                        let dst = unique_child_path(&dst_dir, &file_name);

                        let result = copy_path(&clip.src, &dst);

                        *state.menu_open.lock().unwrap() = false;
                        *state.menu_position.lock().unwrap() = None;
                        ( *state.notify_entity.lock().unwrap(), result )
                    };

                    let _ = result;
                    if let Some(id) = entity_id {
                        cx.notify(id);
                    }
                    window.refresh();
                }),
        );

    // `Popover` anchors its menu to the trigger element. For a context menu we want the anchor
    // to be the mouse click location, so we render an (invisible) absolute trigger at that point.
    let (trigger_left, trigger_top) = menu_position
        .map(|p| (p.x, p.y))
        .unwrap_or((px(0.), px(0.)));

    div()
        .absolute()
        .inset_0()
        .occlude()
        .child(
            // The popover menu positions itself relative to the trigger in *normal flow*.
            // To anchor at an arbitrary point, position the entire popover at the click point
            // and use a 1x1 trigger.
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
}

// Note: filesystem scanning moved out of render into `start_scan(...)`.

fn unique_child_path(parent: &Path, file_name: &str) -> PathBuf {
    // If dst exists, append " (n)".
    let mut candidate = parent.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.to_string());

    let ext = Path::new(file_name)
        .extension()
        .map(|e| e.to_string_lossy().to_string());

    for i in 1..=999u32 {
        let mut name = format!("{} ({})", stem, i);
        if let Some(ext) = &ext {
            name.push('.');
            name.push_str(ext);
        }
        candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    candidate
}

fn copy_path(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        copy_dir_recursive(src, dst)
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    let Ok(read_dir) = fs::read_dir(src) else {
        return Ok(());
    };

    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(file_name);

        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}
