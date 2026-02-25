//! User Actions
//!
//! This module contains action handlers triggered by user interactions.
//!
//! ## Action Pattern
//!
//! Actions are typically:
//! 1. Triggered by UI events (button clicks, menu selections, etc.)
//! 2. Update global state
//! 3. Trigger re-renders via notify_file_browser
//! 4. May spawn async tasks for long-running operations

use std::path::PathBuf;

use gpui::Window;

use crate::scan;
use crate::state::FileBrowserState;

/// Refreshes the current directory tree
///
/// Re-scans the current root directory to pick up any changes.
pub fn refresh(window: &mut Window, cx: &mut gpui::App) {
    let root = {
        let state = cx.global::<FileBrowserState>();
        state.model.read(cx).root.clone()
    };

    scan::start_scan(root, window, cx);
}

/// Changes the root directory and triggers a new scan
///
/// Also clears selection and context menu state.
pub fn set_root_and_rescan(new_root: PathBuf, window: &mut Window, cx: &mut gpui::App) {
    let model = cx.global::<FileBrowserState>().model.clone();
    model.update(cx, |model, cx| {
        model.root = new_root;
        model.selected_path = None;
        model.context_path = None;
        model.menu_open = false;
        model.menu_position = None;
        cx.notify();
    });

    refresh(window, cx);
}

/// Opens a system dialog to select a new root directory
///
/// Uses gpui's built-in path prompt to let users pick a directory.
/// If a directory is selected, triggers a rescan.
pub fn prompt_for_root(window: &mut Window, cx: &mut gpui::App) {
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
                    set_root_and_rescan(path, window, cx);
                }
            })
            .ok();
        })
        .detach();
}
