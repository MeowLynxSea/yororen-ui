//! File System Scanning
//!
//! This module handles asynchronous directory scanning for the file browser.
//!
//! ## Key Concepts
//!
//! - **Background Scanning**: Directory scanning runs in the background using gpui's async runtime
//! - **Generation Tracking**: A generation counter detects stale scans (when root changes during scan)
//! - **Incremental Updates**: Tree nodes are updated incrementally as directories are scanned
//! - **Yield Points**: The scanner yields between directories to keep UI responsive

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::Window;

use yororen_ui::component::{ArcTreeNode, TreeCheckedState, TreeNode};

use crate::state::FileBrowserState;

/// Recursively updates children of a tree node by its ID
///
/// Used to insert scanned directory contents into the tree structure.
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

/// Reads directory contents and creates tree nodes
///
/// Returns a vector of (TreeNode, Option<PathBuf>) pairs:
/// - TreeNode: The UI representation of the file/directory
/// - Option<PathBuf>: Some(path) if it's a directory (for further scanning), None otherwise
///
/// Directories are sorted first, then files, both alphabetically.
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

/// Starts an asynchronous directory scan from the given root path
///
/// This function:
/// 1. Increments the generation counter (to detect stale scans)
/// 2. Sets is_scanning to true
/// 3. Spawns a background task that scans directories up to max_depth (3)
/// 4. Updates tree nodes incrementally as directories are scanned
/// 5. Yields between directories to keep UI responsive
///
/// The generation counter ensures that if the root changes during scanning,
/// older scan results are discarded.
pub fn start_scan(root: PathBuf, window: &mut Window, cx: &mut gpui::App) {
    let model = cx.global::<FileBrowserState>().model.clone();
    let generation = model.update(cx, |model, cx| {
        model.scan_generation = model.scan_generation.wrapping_add(1);
        model.is_scanning = true;
        model.tree_nodes.clear();
        cx.notify();
        model.scan_generation
    });

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
                    let model = cx.global::<FileBrowserState>().model.clone();
                    model.update(cx, |model, cx| {
                        if model.scan_generation != generation {
                            return;
                        }

                        if depth == 0 {
                            model.tree_nodes = children;
                        } else {
                            let _ = set_children_by_id(&mut model.tree_nodes, &dir_id, children);
                        }
                        cx.notify();
                    });
                });

                // Yield between directories so scrolling remains responsive.
                cx.background_executor().timer(Duration::from_millis(8)).await;

                for child_dir in next_dirs {
                    stack.push((child_dir, depth + 1));
                }
            }

            let _ = cx.update(|_window, cx| {
                let model = cx.global::<FileBrowserState>().model.clone();
                model.update(cx, |model, cx| {
                    if model.scan_generation != generation {
                        return;
                    }

                    model.is_scanning = false;
                    cx.notify();
                });
            });
        })
        .detach();
}
