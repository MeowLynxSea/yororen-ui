use gpui::{EntityId, Global, Pixels, Point};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::clipboard::FileClipboard;

pub struct FileBrowserState {
    pub root: Arc<Mutex<PathBuf>>,

    pub selected_path: Arc<Mutex<Option<PathBuf>>>,
    pub context_path: Arc<Mutex<Option<PathBuf>>>,

    pub clipboard: Arc<Mutex<Option<FileClipboard>>>,

    pub menu_open: Arc<Mutex<bool>>,
    pub menu_position: Arc<Mutex<Option<Point<Pixels>>>>,

    // Cached tree nodes built from filesystem scanning.
    pub tree_nodes: Arc<Mutex<Vec<yororen_ui::component::TreeNode>>>,
    pub is_scanning: Arc<Mutex<bool>>,
    pub scan_generation: Arc<Mutex<u64>>,

    pub notify_entity: Arc<Mutex<Option<EntityId>>>,
}

impl Clone for FileBrowserState {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            selected_path: self.selected_path.clone(),
            context_path: self.context_path.clone(),
            clipboard: self.clipboard.clone(),
            menu_open: self.menu_open.clone(),
            menu_position: self.menu_position.clone(),
            tree_nodes: self.tree_nodes.clone(),
            is_scanning: self.is_scanning.clone(),
            scan_generation: self.scan_generation.clone(),
            notify_entity: self.notify_entity.clone(),
        }
    }
}

impl Default for FileBrowserState {
    fn default() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            root: Arc::new(Mutex::new(root)),
            selected_path: Arc::new(Mutex::new(None)),
            context_path: Arc::new(Mutex::new(None)),
            clipboard: Arc::new(Mutex::new(None)),
            menu_open: Arc::new(Mutex::new(false)),
            menu_position: Arc::new(Mutex::new(None)),
            tree_nodes: Arc::new(Mutex::new(Vec::new())),
            is_scanning: Arc::new(Mutex::new(false)),
            scan_generation: Arc::new(Mutex::new(0)),
            notify_entity: Arc::new(Mutex::new(None)),
        }
    }
}

impl Global for FileBrowserState {}

pub fn notify_file_browser(cx: &mut gpui::App) {
    let id = {
        let state = cx.global::<FileBrowserState>();
        *state.notify_entity.lock().unwrap()
    };
    if let Some(id) = id {
        cx.notify(id);
    }
}
