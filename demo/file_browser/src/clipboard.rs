//! yororen-ui Domain Model Pattern
//!
//! This module demonstrates how to structure domain models in yororen-ui applications.
//!
//! ## Domain Model Pattern
//!
//! Keep your domain models (data structures) separate from UI code:
//! - No gpui or yororen-ui imports
//! - Plain Rust structs and enums
//! - Business logic only
//!
//! ## Why Separate Models?
//!
//! Separating models from UI makes code:
//! - Testable: Business logic can be unit tested without UI
//! - Reusable: Models can be used in different contexts
//! - Clean: Clear separation of concerns
//!
//! ## This Pattern in Your App
//!
//! Create a `models.rs` or similar module for your domain types:
//! ```ignore
//! // models.rs
//! pub struct MyEntity { ... }
//! pub enum MyStatus { ... }
//! ```

use std::path::PathBuf;

/// Domain enum - represents clipboard operations
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardOp {
    /// Copy operation - copies file/directory to clipboard
    Copy,
}

/// Domain entity - represents clipboard contents for file operations
///
/// Stores the operation type and source path for copy/paste operations.
#[derive(Clone, Debug)]
pub struct FileClipboard {
    /// The operation to perform
    pub op: ClipboardOp,
    /// Source path of the file/directory
    pub src: PathBuf,
}

