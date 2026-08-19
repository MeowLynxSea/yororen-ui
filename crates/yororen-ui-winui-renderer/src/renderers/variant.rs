//! Variant registry — open extension point for the 3 built-in
//! `ActionVariantKind` values.
//!
//! `core` ships three built-in variants (Neutral / Primary / Danger).
//! Apps and theme packages can register more (e.g. "ghost", "outline",
//! "branded", "destructive-secondary") without modifying `core`. The
//! registry is `Send + Sync` so a `Theme` can hold one in a global.
//!
//! All types in this module are re-exported from
//! `yororen_ui_core::renderer`; the default-renderer keeps thin
//! wrappers for backward-compat with older import paths.

pub use yororen_ui_core::renderer::{
    BuiltinVariantKey, ButtonVariant, GlobalVariantRegistry, TokenVariantStyle, VariantKey,
    VariantRegistry, VariantState, VariantStyle, variant_compose,
};

use std::sync::Arc;

use yororen_ui_core::renderer::ActionVariantKind;

/// Backward-compat alias. Older code uses `super::variant::ActionVariant`;
/// forward to the core enum.
pub type ActionVariant = ActionVariantKind;

#[doc(hidden)]
pub type ComposedVariant = Arc<dyn VariantStyle>;
