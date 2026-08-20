//! Default renderer for [`yororen_ui_core`] headless primitives.
//!
//! This crate holds the *visual* layer of yororen-ui: the 56
//! `XxxRenderer` traits (defined in `yororen-ui-core`), their
//! `TokenXxxRenderer` default implementations, the bundled
//! `system-light.json` / `system-dark.json` themes, and a
//! one-call install helper.
//!
//! Each `renderers/<name>.rs` file is the home of the
//! `TokenXxxRenderer` struct (the default impl). The
//! `XxxRenderer` trait + `XxxRenderState` struct live in
//! `yororen-ui-core/src/renderer/<name>.rs` so the headless
//! `XxxProps::render()` method can look them up via the
//! registry.
//!
//! Three-layer architecture:
//!
//! ```text
//! themes/*.json ──▶ default-renderer ──▶ core ──▶ gpui ce
//! ```
//!
//! Apps depend on this crate when they want a stock look.
//! Third-party renderer crates depend on `yororen-ui-core`
//! and call `cx.register_renderer_arc::<markers::X, dyn XxxRenderer>(...)`
//! at install time.

pub mod animation;
pub mod renderers;
pub mod themes;

pub use renderers::{
    ButtonRenderState, ButtonRenderer, IconButtonRenderState, IconButtonRenderer, LabelRenderState,
    LabelRenderer, ToggleButtonRenderState, ToggleButtonRenderer,
};
pub use themes::{
    install, install_with, register_default_renderers, system_dark, system_for, system_light,
};
pub use yororen_ui_core::theme::Theme;
