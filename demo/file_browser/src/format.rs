use std::path::PathBuf;

use crate::clipboard::{ClipboardOp, FileClipboard};

pub fn path_or_dash(path: &Option<PathBuf>) -> String {
    path.as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "-".to_string())
}

pub fn clipboard_label(clipboard: &Option<FileClipboard>) -> String {
    let Some(clipboard) = clipboard else {
        return "-".to_string();
    };

    match clipboard.op {
        ClipboardOp::Copy => format!("copy: {}", clipboard.src.to_string_lossy()),
    }
}

