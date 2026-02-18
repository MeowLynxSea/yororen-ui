//! File System Operations - Domain Logic
//!
//! This module demonstrates how to structure **business logic** separate from UI code.
//!
//! ## Domain Logic Pattern
//!
//! Keep file system operations (business logic) separate from UI components:
//! - No gpui or yororen-ui imports
//! - Plain Rust functions
//! - Pure file system operations
//!
//! ## Why Separate Logic?
//!
//! Separating business logic from UI makes code:
//! - Testable: Can be unit tested without UI
//! - Reusable: Functions can be used in different contexts
//! - Clean: Clear separation of concerns
//!
//! ## This Pattern in Your App
//!
//! Create a `domain.rs` or similar module for your business logic:
//! ```ignore
//! // domain.rs
//! pub fn calculate_total(items: &[Item]) -> f64 { ... }
//! pub fn validate_input(input: &str) -> Result<(), Error> { ... }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

/// Generates a unique file path by appending a number if the file exists
///
/// If `file_name` doesn't exist in `parent`, returns that path.
/// Otherwise, tries `file_name (1)`, `file_name (2)`, etc. up to 999.
///
/// # Arguments
/// * `parent` - The parent directory
/// * `file_name` - The desired file name
///
/// # Returns
/// A unique PathBuf that doesn't conflict with existing files
pub fn unique_child_path(parent: &Path, file_name: &str) -> PathBuf {
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

/// Copies a file or directory to a destination path
///
/// Handles both files and directories:
/// - For files: copies the file content
/// - For directories: recursively copies all contents
///
/// # Arguments
/// * `src` - Source file or directory path
/// * `dst` - Destination path
///
/// # Returns
/// `Ok(())` on success, or an I/O error
pub fn copy_path(src: &Path, dst: &Path) -> std::io::Result<()> {
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

/// Recursively copies a directory and all its contents
///
/// This is an internal helper function used by `copy_path`.
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

