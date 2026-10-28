//! `ButtonRenderer` trait — the data contract that every
//! third-party renderer (default, winui, …) implements to
//! describe a `Button`'s visual.
//!
//! Lives in core (not the default-renderer) so headless
//! `ButtonProps::render` can call `compose`. The trait surface
//! is intentionally minimal: a single `compose` method that
//! takes the full `ButtonProps` and returns a fully-built
//! `Stateful<Div>` (visuals + children + hover / active + id +
//! focus). All token reads, palette lookups and styling
//! choices live inside the renderer — core does not know
//! about `bg` / `fg` / `border` / etc.
//!
//! `ButtonRenderState` is provided as a convenience projection
//! of `ButtonProps` for renderers that want to share helpers
//! across the suite (e.g. token-based vs. WinUI), but it
//! is not part of the trait surface.

use std::any::Any;
use std::sync::Arc;

use gpui::{App, Div, FocusHandle, Stateful};

use crate::headless::button::ButtonProps;
use crate::renderer::variant::VariantStyle;

/// Projection of `ButtonProps` used by built-in renderers when
/// they want to factor out helpers (bg / fg / border / etc.).
/// Not part of the `ButtonRenderer` trait surface.
#[derive(Clone, Debug, Default)]
pub struct ButtonRenderState {
    pub variant: ActionVariantKind,
    pub disabled: bool,
    pub is_rtl: bool,
    /// `true` if the user supplied `.bg(...)` on the builder.
    pub has_custom_bg: bool,
    /// `true` if the user supplied `.hover_bg(...)` on the
    /// builder.
    pub has_custom_hover_bg: bool,
    /// Pre-resolved custom variant from the global
    /// `VariantRegistry`. When `Some`, the renderer should
    /// delegate color decisions to the contained
    /// `VariantStyle` instead of reading from theme paths.
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

pub use crate::renderer::variant::ActionVariantKind;

/// Renderer for the `Button` component. Implementations decide
/// what the button looks like in every state.
///
/// Default: `TokenButtonRenderer` (in the default-renderer
/// crate). Theme packages / renderer crates override this by
/// registering their own `ButtonRenderer` impl via
/// `cx.register_renderer_arc::<ButtonMarker, dyn ButtonRenderer>(…)`.
pub trait ButtonRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for a button.
    fn compose(&self, props: &ButtonProps, focus_handle: &FocusHandle, cx: &App) -> Stateful<Div>;
}
