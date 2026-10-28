//! 58 component renderers grouped by visual category.
//!
//! Each submodule implements the `XxxRenderer` traits from
//! `yororen_ui_default_renderer::renderers` with brutalist visual
//! values: sharp corners, thick black borders, hard offset shadows,
//! monospace typography, and high-contrast solid colors.
//!
//! Colors are read from the theme (so light/dark variants work);
//! geometry is hardcoded in the `style` module so the 58
//! renderers stay in stylistic lockstep.

pub mod actions;
pub mod controls;
pub mod display;
pub mod inputs;
pub mod lists;
pub mod notifications;
pub mod overlays;
pub mod scroll;
pub mod surfaces;
