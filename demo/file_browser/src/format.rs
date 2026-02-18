//! Formatting Utilities
//!
//! This module contains helper functions for formatting data for display in the UI.

use std::path::PathBuf;

use crate::clipboard::{ClipboardOp, FileClipboard};

/// Formats a path for display, or "-" if None
pub fn path_or_dash(path: &Option<PathBuf>) -> String {
    path.as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "-".to_string())
}

/// Formats clipboard contents for display, or "-" if empty
pub fn clipboard_label(clipboard: &Option<FileClipboard>) -> String {
    let Some(clipboard) = clipboard else {
        return "-".to_string();
    };

    match clipboard.op {
        ClipboardOp::Copy => format!("copy: {}", clipboard.src.to_string_lossy()),
    }
}

