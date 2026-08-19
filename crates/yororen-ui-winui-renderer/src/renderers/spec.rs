//! Generic spec types used by component Renderers.
//!
//! These exist so a Renderer method can return a structured value instead
//! of a primitive (e.g. `Edges<Pixels>` instead of four `Pixels` args).
//! The fields are intentionally minimal: a theme package can always
//! reach into the underlying `Theme` for richer configuration.

use std::any::Any;

use gpui::{Hsla, Pixels};

/// Marker trait that every component's `XxxRenderState` implements.
///
/// Used as the keying type in `RendererRegistry`'s internal `HashMap`:
/// the registry stores renderers under `TypeId::of::<S>()`, and every
/// render path that wants its renderer looks it up by `S`.
///
/// The blanket impl below makes every `'static + Send + Sync` type a
/// `RenderState` automatically — so the 55 `XxxRenderState` structs
/// scattered across `renderer/<component>.rs` don't need a hand-written
/// `impl RenderState for XxxRenderState {}` each. They are composed of
/// primitives + `Arc<dyn VariantStyle>` and therefore already satisfy
/// the bounds.
pub trait RenderState: Any + 'static + Send + Sync {}

impl<T> RenderState for T where T: Any + 'static + Send + Sync {}

/// Four-sided edges, with each side independent.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Copy> Edges<T> {
    /// All four sides set to the same value.
    pub fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Symmetric horizontal/vertical (e.g. padding-x, padding-y).
    pub fn symmetric(horizontal: T, vertical: T) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// Border spec.
#[derive(Clone, Copy, Debug)]
pub struct BorderSpec {
    pub width: Pixels,
    pub color: Hsla,
}

impl BorderSpec {
    pub fn new(width: impl Into<Pixels>, color: Hsla) -> Self {
        Self {
            width: width.into(),
            color,
        }
    }
}

/// Shadow spec. Stored as a small inline representation; a theme package
/// can layer this on top of the GPUI `BoxShadow` builder at render time.
#[derive(Clone, Copy, Debug)]
pub struct ShadowSpec {
    pub blur: Pixels,
    pub offset_y: Pixels,
    pub color: Hsla,
}

impl ShadowSpec {
    pub fn new(blur: impl Into<Pixels>, offset_y: impl Into<Pixels>, color: Hsla) -> Self {
        Self {
            blur: blur.into(),
            offset_y: offset_y.into(),
            color,
        }
    }
}

/// Where a control should place its leading icon relative to its label.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum IconPosition {
    #[default]
    Leading,
    Trailing,
    Only,
}
