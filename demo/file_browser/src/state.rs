//! yororen-ui Global State Pattern
//!
//! This module demonstrates the **recommended pattern** for managing application state
//! in yororen-ui/gpui applications.
//!
//! ## Why This Pattern?
//!
//! In yororen-ui, components are often rendered from different contexts (closures).
//! Standard Rust ownership doesn't allow multiple parts of the code to mutate the same data.
//! The `Arc<Mutex<T>>` pattern solves this:
//!
//! - **Arc** (Atomic Reference Counted): Allows multiple owners to share data
//! - **Mutex** (Mutual Exclusion): Ensures only one part can mutate at a time
//!
//! ## State Update Flow (Important!)
//!
//! ```ignore
//! 1. Component reads state: let value = *state.field.lock().unwrap();
//! 2. Component modifies state: *state.field.lock().unwrap() = new_value;
//! 3. Component triggers re-render: cx.notify(entity_id);
//! 4. gpui re-renders the component that owns the entity
//! ```
//!
//! ## This Pattern in Your App
//!
//! To add global state to your yororen-ui application:
//! 1. Define a struct with `Arc<Mutex<T>>` fields
//! 2. Implement `Clone` (automatically via derive or manually)
//! 3. Implement `Default` for initial state
//! 4. Implement `Global` trait
//! 5. Set via `cx.set_global(YourState::default())` in main()

use gpui::{EntityId, Global, Pixels, Point};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::clipboard::FileClipboard;

/// Standard yororen-ui global state struct
///
/// All fields follow the Arc<Mutex<T>> pattern for safe shared mutation.
/// This struct is accessible from any component via `cx.global::<FileBrowserState>()`.
pub struct FileBrowserState {
    /// Current root directory being browsed
    pub root: Arc<Mutex<PathBuf>>,

    /// Currently selected file/directory path
    pub selected_path: Arc<Mutex<Option<PathBuf>>>,

    /// Path on which context menu was triggered
    pub context_path: Arc<Mutex<Option<PathBuf>>>,

    /// Clipboard for file operations (copy/paste)
    pub clipboard: Arc<Mutex<Option<FileClipboard>>>,

    /// Whether context menu is currently open
    pub menu_open: Arc<Mutex<bool>>,

    /// Position where context menu should appear
    pub menu_position: Arc<Mutex<Option<Point<Pixels>>>>,

    /// Cached tree nodes built from filesystem scanning
    pub tree_nodes: Arc<Mutex<Vec<yororen_ui::component::TreeNode>>>,

    /// Whether a scan is currently in progress
    pub is_scanning: Arc<Mutex<bool>>,

    /// Generation counter to detect stale scan operations
    pub scan_generation: Arc<Mutex<u64>>,

    /// Notification system: stores entity_id of the component that should receive re-render notifications
    pub notify_entity: Arc<Mutex<Option<EntityId>>>,
}

/// Required for sharing state across components
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

/// Provides initial state when app starts
impl Default for FileBrowserState {
    fn default() -> Self {
        // Use current working directory as default root
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

/// REQUIRED: Makes this type available as global state
///
/// Only types implementing `Global` can be accessed via `cx.global::<T>()`
impl Global for FileBrowserState {}

/// Notifies the FileBrowserApp to re-render
///
/// This function retrieves the stored entity_id from global state
/// and calls `cx.notify()` to trigger a re-render.
pub fn notify_file_browser(cx: &mut gpui::App) {
    let id = {
        let state = cx.global::<FileBrowserState>();
        *state.notify_entity.lock().unwrap()
    };
    if let Some(id) = id {
        cx.notify(id);
    }
}
