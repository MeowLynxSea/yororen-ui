use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::Window;

use yororen_ui::component::{ArcTreeNode, TreeCheckedState, TreeNode};

use crate::state::{notify_file_browser, FileBrowserState};

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
            checked: TreeCheckedState::Unchecked,
            depth: 0,
            has_children: is_dir,
        };
        out.push((node, is_dir.then_some(path)));
    }

    out
}

pub fn start_scan(root: PathBuf, window: &mut Window, cx: &mut gpui::App) {
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

