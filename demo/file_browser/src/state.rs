//! yororen-ui Global State Management (Entity-based)
//!
//! This demo stores app state in a `gpui::Entity<FileBrowserModel>`.
//!
//! Notes:
//! - `Entity::update(...)` does not automatically trigger a redraw in `gpui-ce 0.3`.
//!   Call `cx.notify()` inside the update closure after mutating state.

use std::path::PathBuf;

use gpui::{App, AppContext, Entity, Global, Pixels, Point};

use crate::clipboard::FileClipboard;

#[derive(Clone)]
pub struct FileBrowserState {
    pub model: Entity<FileBrowserModel>,
}

impl FileBrowserState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            model: cx.new(|_| FileBrowserModel::default()),
        }
    }
}

impl Global for FileBrowserState {}

pub struct FileBrowserModel {
    pub root: PathBuf,
    pub selected_path: Option<PathBuf>,
    pub context_path: Option<PathBuf>,
    pub clipboard: Option<FileClipboard>,
    pub menu_open: bool,
    pub menu_position: Option<Point<Pixels>>,
    pub tree_nodes: Vec<yororen_ui::component::TreeNode>,
    pub is_scanning: bool,
    pub scan_generation: u64,
}

impl Default for FileBrowserModel {
    fn default() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            root,
            selected_path: None,
            context_path: None,
            clipboard: None,
            menu_open: false,
            menu_position: None,
            tree_nodes: Vec::new(),
            is_scanning: false,
            scan_generation: 0,
        }
    }
}
