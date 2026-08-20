//! `RendererRegistry` — the core's per-process registry of
//! component renderers, looked up by marker type at render time.
//!
//! The v0.3 split moves the registry out of the renderer crate
//! and into core: a renderer crate (built-in or third-party)
//! just calls `cx.register_renderer_arc::<headless::Button>(r)`
//! at install time, and component code (or a renderer-specific
//! sugar like `DefaultButton::default_render`) calls
//! `cx.renderer_arc::<headless::Button, dyn ButtonRenderer>()`
//! at render time.
//!
//! ## Storage model
//!
//! ```text
//! HashMap<TypeId, Arc<dyn Any + Send + Sync>>
//!           │      │
//!           │      └─ wraps the caller-supplied `Arc<R>` in
//!           │         another Arc, so the value stored inside
//!           │         the outer Arc is itself an `Arc<R>`.
//!           │         Downcast to `Arc<R>` recovers the
//!           │         original typed renderer reference.
//!           └─ keyed by the marker type T passed at register.
//! ```
//!
//! The double-Arc wrap is the same trick the v0.4
//! `RendererRegistry` in `yororen-ui-renderer` used: it lets us
//! store trait objects (unsized) and still downcast by Sized
//! key. The marker `T` is the headless component (e.g.
//! `headless::Button`); the trait `R` is the renderer's own
//! contract (e.g. `dyn ButtonRenderer`).
//!
//! ## Marker trait
//!
//! [`RendererMarker`] is the minimal trait every component
//! marker (the `pub struct Button;` in `headless::button`, etc.)
//! implements. The registry itself does not constrain the
//! marker; the trait is for documentation and future-proofing
//! only — you could pass any `'static` type as the key.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use gpui::Global;

/// Marker trait for component types that can have a renderer
/// registered against them. The 56 unit structs in
/// `yororen-ui-core::headless::*` (one per component) implement
/// this trait; user code may add markers for custom components.
pub trait RendererMarker: 'static {}

/// Process-global registry of renderers, keyed by marker
/// type.
///
/// Register at install time:
///
/// ```ignore
/// cx.register_renderer_arc::<headless::Button>(
///     Arc::new(TokenButtonRenderer) as Arc<dyn ButtonRenderer>
/// );
/// ```
///
/// Retrieve at render time:
///
/// ```ignore
/// let r: &Arc<dyn ButtonRenderer> = cx
///     .renderer_arc::<headless::Button, dyn ButtonRenderer>()
///     .expect("ButtonRenderer registered");
/// ```
#[derive(Default)]
pub struct RendererRegistry {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Global for RendererRegistry {}

impl std::fmt::Debug for RendererRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererRegistry")
            .field("count", &self.map.len())
            .finish()
    }
}

impl RendererRegistry {
    /// Empty registry. No renderers registered. `Default` is
    /// also `new()`.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Register a typed renderer for marker `T`. The renderer
    /// is the trait object the renderer crate wants the
    /// component to dispatch through (e.g. `dyn ButtonRenderer`).
    /// Subsequent calls with the same `T` overwrite (last-wins)
    /// — useful for theme switching at runtime.
    pub fn register_arc<T: RendererMarker, R: ?Sized + Send + Sync + 'static>(
        &mut self,
        r: Arc<R>,
    ) {
        // Double-Arc wrap. The outer Arc<dyn Any> contains the
        // caller's `Arc<R>` as its value, so downcasting the
        // outer Arc to `Arc<R>` (Sized) recovers the typed
        // renderer reference.
        let outer: Arc<dyn Any + Send + Sync> = Arc::new(r);
        self.map.insert(TypeId::of::<T>(), outer);
    }

    /// Borrow the renderer registered for marker `T`, downcast
    /// to `Arc<R>`. Returns `None` if no renderer is registered
    /// for `T`, or the registered one is not of the requested
    /// concrete `R`.
    ///
    /// `R` is typically `dyn SomeRenderer`; the downcast works
    /// because the setter wrapped the renderer in
    /// `Arc<Arc<dyn SomeRenderer>>` and `Arc<dyn SomeRenderer>`
    /// is `'static + Sized`.
    pub fn renderer_arc<T: RendererMarker, R: ?Sized + Send + Sync + 'static>(
        &self,
    ) -> Option<&Arc<R>> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.downcast_ref::<Arc<R>>())
    }

    /// True iff a renderer is registered for marker `T`. Cheap
    /// — single `HashMap` lookup.
    pub fn contains<T: RendererMarker>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    /// Number of registered renderers. Useful for tests and
    /// debug assertions ("did theme install register everything
    /// it claimed to?").
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True iff no renderers are registered.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMarkerA;
    impl RendererMarker for TestMarkerA {}
    struct TestMarkerB;
    impl RendererMarker for TestMarkerB {}

    trait MyTrait: Any + Send + Sync {
        fn name(&self) -> &str;
    }
    struct A;
    impl MyTrait for A {
        fn name(&self) -> &str {
            "A"
        }
    }
    struct B;
    impl MyTrait for B {
        fn name(&self) -> &str {
            "B"
        }
    }

    #[test]
    fn empty_registry() {
        let r = RendererRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert!(!r.contains::<TestMarkerA>());
    }

    #[test]
    fn register_then_retrieve() {
        let mut r = RendererRegistry::new();
        let arc: Arc<dyn MyTrait> = Arc::new(A);
        r.register_arc::<TestMarkerA, dyn MyTrait>(arc);

        let got: &Arc<dyn MyTrait> = r.renderer_arc::<TestMarkerA, dyn MyTrait>().unwrap();
        assert_eq!(got.name(), "A");
        assert_eq!(r.len(), 1);
        assert!(r.contains::<TestMarkerA>());
        assert!(!r.contains::<TestMarkerB>());
    }

    #[test]
    fn last_wins_on_overwrite() {
        let mut r = RendererRegistry::new();
        r.register_arc::<TestMarkerA, dyn MyTrait>(Arc::new(A) as Arc<dyn MyTrait>);
        r.register_arc::<TestMarkerA, dyn MyTrait>(Arc::new(B) as Arc<dyn MyTrait>);

        let got: &Arc<dyn MyTrait> = r.renderer_arc::<TestMarkerA, dyn MyTrait>().unwrap();
        assert_eq!(got.name(), "B");
        // Still one slot — overwrite, not append.
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn different_markers_coexist() {
        let mut r = RendererRegistry::new();
        r.register_arc::<TestMarkerA, dyn MyTrait>(Arc::new(A) as Arc<dyn MyTrait>);
        r.register_arc::<TestMarkerB, dyn MyTrait>(Arc::new(B) as Arc<dyn MyTrait>);
        assert_eq!(r.len(), 2);
        assert_eq!(
            r.renderer_arc::<TestMarkerA, dyn MyTrait>().unwrap().name(),
            "A"
        );
        assert_eq!(
            r.renderer_arc::<TestMarkerB, dyn MyTrait>().unwrap().name(),
            "B"
        );
    }

    #[test]
    fn wrong_concrete_type_returns_none() {
        let mut r = RendererRegistry::new();
        r.register_arc::<TestMarkerA, dyn MyTrait>(Arc::new(A) as Arc<dyn MyTrait>);
        // Try to downcast to a different type — should miss.
        let wrong: Option<&Arc<String>> = r.renderer_arc::<TestMarkerA, String>();
        assert!(wrong.is_none());
        // The original retrieval still works.
        let right: Option<&Arc<dyn MyTrait>> = r.renderer_arc::<TestMarkerA, dyn MyTrait>();
        assert!(right.is_some());
    }

    #[test]
    fn missing_marker_returns_none() {
        let r = RendererRegistry::new();
        assert!(r.renderer_arc::<TestMarkerA, dyn MyTrait>().is_none());
    }

    #[test]
    fn default_is_empty() {
        let r = RendererRegistry::default();
        assert!(r.is_empty());
    }
}
