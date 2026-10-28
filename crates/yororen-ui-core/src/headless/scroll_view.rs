//! Headless `scroll_view` — a generic scrollable container with
//! renderer-drawn scrollbars, wrapping `gpui::ScrollHandle`.
//!
//! ## Why a controller and not raw `gpui::ScrollHandle`?
//!
//! Same reasoning as [`crate::headless::virtual_list`]'s
//! [`VirtualListController`](crate::headless::virtual_list::VirtualListController):
//! the underlying `ScrollHandle` is `Rc<RefCell<…>>`, so a thin
//! `&self` wrapper gives the caller `scroll_to / scroll_by /
//! scroll_to_item / is_at_top / is_at_bottom …` without repeating
//! clamping arithmetic at every call site. Notably
//! `ScrollHandle::set_offset` does **not** clamp — clamping happens
//! during the owning element's next prepaint — so the controller
//! clamps eagerly against `max_offset()` to make `scroll_by`
//! immediately idempotent at the boundaries.
//!
//! ## Child-bounds contract for `scroll_to_item`
//!
//! `ScrollHandle::bounds_for_item` / `scroll_to_item` only know the
//! **direct children** of the `overflow_scroll` element that called
//! `.track_scroll(&handle)` (gpui collects child layout bounds during
//! that element's prepaint). Scrolling to a nested descendant is the
//! caller's job (walk your own layout, then `scroll_to(point)`).
//!
//! ## Children ownership
//!
//! Unlike the closure-driven virtual list, the scroll view's content
//! is a plain `Vec<AnyElement>` accumulated through `.child(...)`.
//! `render(cx)` therefore **consumes** the props (Variant B) so the
//! children can be moved into the renderer's content area.

use std::sync::{Arc, Mutex};

use gpui::{
    App, Bounds, Div, ElementId, InteractiveElement, Pixels, Point, ScrollHandle, Size, Stateful,
    Window, point, px, size,
};

/// A `&self` handle over a `gpui::ScrollHandle` — the caller stores
/// one of these on their view (or inside another component's state)
/// and uses the `scroll_to*` / `scroll_by` family to drive the scroll
/// position across frames.
///
/// Cheap to clone (the inner handle is `Rc<RefCell<…>>`).
///
/// Note on `drag_anchor`: the scrollbar-thumb drag anchor must live
/// here (the caller-owned controller), NOT in the renderer's frame
/// closure. Renderers rebuild their element every frame — and they
/// `window.refresh()` after each thumb move — so a frame-local
/// anchor is reset to empty on every move, which makes the thumb
/// stick to the cursor and scroll zero pixels.
#[derive(Clone, Debug)]
pub struct ScrollViewController {
    handle: ScrollHandle,
    /// Thumb-drag bookkeeping (see [`ThumbDragState`]). Lives on the
    /// controller — not the renderer closure — because renderers
    /// rebuild their element every frame (and `refresh()` after each
    /// move), so frame-local state is wiped between drag moves.
    thumb_drag: Arc<Mutex<Option<ThumbDragState>>>,
}

/// State captured when a scrollbar-thumb drag starts: the absolute
/// window-space cursor position (the delta on each move is measured
/// against this, since gpui's `on_drag` element-relative offset is
/// invalidated as the thumb shifts under the pointer) plus the full
/// scroll offset at drag start.
#[derive(Clone, Copy, Debug)]
pub struct ThumbDragState {
    pub anchor: Point<Pixels>,
    pub start_offset: Point<Pixels>,
}

impl ScrollViewController {
    /// Mint a fresh controller. Bind it to a `ScrollViewProps` via
    /// [`scroll_view`]; the renderer attaches it to the content
    /// area with `track_scroll`.
    pub fn new() -> Self {
        Self {
            handle: ScrollHandle::new(),
            thumb_drag: Arc::new(Mutex::new(None)),
        }
    }

    /// Snapshot the inner `gpui::ScrollHandle` — used by renderers
    /// for `track_scroll`.
    pub fn handle(&self) -> ScrollHandle {
        self.handle.clone()
    }

    /// Current scroll offset (≤ 0 on both axes; more negative the
    /// further down/right the content is scrolled).
    pub fn offset(&self) -> Point<Pixels> {
        self.handle.offset()
    }

    /// Maximum scroll magnitude per axis (i.e. `content_size -
    /// viewport_size`). Zero on an axis means it cannot scroll.
    /// Valid only after the tracked element's first prepaint.
    pub fn max_offset(&self) -> Size<Pixels> {
        self.handle.max_offset().into()
    }

    /// Size of the visible region (the tracked element's bounds).
    /// Zero before the first prepaint.
    pub fn viewport_size(&self) -> Size<Pixels> {
        self.handle.bounds().size
    }

    /// Total laid-out size of the scrolled content
    /// (`max_offset + viewport`). Zero before the first prepaint.
    pub fn content_size(&self) -> Size<Pixels> {
        let vp = self.viewport_size();
        let max = self.max_offset();
        size(vp.width + max.width, vp.height + max.height)
    }

    /// Eagerly clamped `set_offset` — `gpui::ScrollHandle::set_offset`
    /// itself defers clamping to the next prepaint, so this wrapper
    /// keeps programmatic scrolls sane at the boundaries.
    pub fn scroll_to(&self, offset: Point<Pixels>) {
        self.handle.set_offset(self.clamp_offset(offset));
    }

    /// Scroll by a delta (negative y scrolls down, matching the
    /// offset convention). Clamped so repeated calls stick at the
    /// edges instead of accumulating past them.
    pub fn scroll_by(&self, delta: Point<Pixels>) {
        let o = self.offset();
        self.scroll_to(point(o.x + delta.x, o.y + delta.y));
    }

    /// Scroll to the very top (offset y = 0).
    pub fn scroll_to_top(&self) {
        let o = self.offset();
        self.scroll_to(point(o.x, Pixels::ZERO));
    }

    /// Scroll to the very bottom. Delegates to
    /// `ScrollHandle::scroll_to_bottom`, whose flag resolves during
    /// the next prepaint (and correctly re-clamps after resizes).
    pub fn scroll_to_bottom(&self) {
        self.handle.scroll_to_bottom();
    }

    /// Scroll to the left edge (offset x = 0).
    pub fn scroll_to_left(&self) {
        let o = self.offset();
        self.scroll_to(point(Pixels::ZERO, o.y));
    }

    /// Scroll to the right edge (offset x = -max_offset.width).
    pub fn scroll_to_right(&self) {
        let o = self.offset();
        let max = self.max_offset();
        self.scroll_to(point(-max.width, o.y));
    }

    /// Scroll the minimal amount so that direct child `ix` is fully
    /// visible. See the module docs — only direct children of the
    /// tracked element are addressable.
    pub fn scroll_to_item(&self, ix: usize) {
        self.handle.scroll_to_item(ix);
    }

    /// Scroll so that direct child `ix` sits at the top of the
    /// viewport.
    pub fn scroll_to_top_of_item(&self, ix: usize) {
        self.handle.scroll_to_top_of_item(ix);
    }

    /// Layout bounds (window coordinates) of direct child `ix`, if
    /// it has been laid out.
    pub fn bounds_for_item(&self, ix: usize) -> Option<Bounds<Pixels>> {
        self.handle.bounds_for_item(ix)
    }

    /// Number of direct children the handle has measured.
    pub fn children_count(&self) -> usize {
        self.handle.children_count()
    }

    /// True when the viewport shows the top of the content (also
    /// true when the content cannot scroll at all).
    ///
    /// The comparisons carry a half-pixel tolerance because scroll
    /// offsets regularly land on negative zero (`-0.0`), which
    /// `Pixels`' ordering treats as strictly below zero.
    pub fn is_at_top(&self) -> bool {
        self.offset().y >= -px(0.5)
    }

    /// True when the viewport shows the end of the content (also
    /// true when the content cannot scroll at all).
    pub fn is_at_bottom(&self) -> bool {
        self.offset().y <= -self.max_offset().height + px(0.5)
    }

    /// True when the viewport shows the left edge of the content.
    pub fn is_at_left(&self) -> bool {
        self.offset().x >= -px(0.5)
    }

    /// True when the viewport shows the right edge of the content.
    pub fn is_at_right(&self) -> bool {
        self.offset().x <= -self.max_offset().width + px(0.5)
    }

    // -- Scrollbar-thumb drag anchor -------------------------------
    //
    // The renderers record the mouse position when a thumb drag
    // starts and measure the cursor delta against it on every move.
    // gpui's `on_drag` reports a cursor offset *relative to the
    // element*, which is invalidated as soon as the thumb (and hence
    // the element) shifts under the pointer; the delta must instead
    // use absolute window coordinates — captured per drag, not per
    // frame.

    /// Capture the absolute cursor position + current offset when a
    /// thumb drag starts. Called from the thumb's `on_drag`.
    pub fn begin_thumb_drag(&self, position: Point<Pixels>) {
        *self.thumb_drag.lock().unwrap() = Some(ThumbDragState {
            anchor: position,
            start_offset: self.offset(),
        });
    }

    /// The drag-start state, if a thumb drag is in progress.
    pub fn thumb_drag_state(&self) -> Option<ThumbDragState> {
        *self.thumb_drag.lock().unwrap()
    }

    /// Clear the drag state when the drag ends (mouse up / cancel).
    pub fn end_thumb_drag(&self) {
        *self.thumb_drag.lock().unwrap() = None;
    }

    fn clamp_offset(&self, offset: Point<Pixels>) -> Point<Pixels> {
        let max = self.max_offset();
        point(
            offset.x.clamp(-max.width, Pixels::ZERO),
            offset.y.clamp(-max.height, Pixels::ZERO),
        )
    }
}

impl Default for ScrollViewController {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether the renderer draws scrollbars. `Always` shows the thumb
/// whenever the corresponding axis is scrollable (hover effects are
/// up to the renderer); `Never` disables the overlay entirely —
/// wheel / keyboard / programmatic scrolling still work.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarVisibility {
    #[default]
    Always,
    Never,
}

/// Fired on user-driven scrolls (wheel, thumb drag, keyboard). The
/// arguments are `(offset, max_offset)` — enough to derive position
/// ratio and at-edge predicates without reading the controller
/// again.
pub type ScrollViewScrollCallback = Arc<dyn Fn(Point<Pixels>, Size<Pixels>, &mut Window, &mut App)>;

/// A snapshot of the data + control layer for a scroll view.
/// Constructed by [`scroll_view`], mutated through builder methods,
/// and consumed by `.render(cx)` (which hands it — children and all —
/// to the `ScrollViewRenderer`).
pub struct ScrollViewProps {
    pub id: ElementId,
    pub controller: ScrollViewController,
    /// Whether the horizontal axis scrolls. `false` by default.
    pub scroll_x: bool,
    /// Whether the vertical axis scrolls. `true` by default.
    pub scroll_y: bool,
    pub scrollbar_visibility: ScrollbarVisibility,
    /// Draw the renderer's themed frame (surface bg, border,
    /// radius). `false` by default — WinUI's `ScrollViewer` is a
    /// bare content control, and bare is what embedders
    /// (Listbox/ComboBox dropdowns, TextArea, …) want.
    pub framed: bool,
    /// Cap the content area's height. `None` (default) sizes to
    /// content / the caller's outer constraint. `Some(px)` makes
    /// the view grow with its children up to the cap and scroll
    /// beyond — the dropdown-menu shape. Applied to the content
    /// div (not the outer shell) so the scrollbar overlay keeps
    /// tracking the clamped height.
    pub max_content_height: Option<Pixels>,
    /// Gap between the content area's direct children — used by
    /// embedders whose items are direct children of the scroll
    /// area (dropdown menus), where `scroll_to_item` addressing
    /// requires exactly that structure.
    pub gap: Option<Pixels>,
    pub children: Vec<gpui::AnyElement>,
    pub on_scroll: Option<ScrollViewScrollCallback>,
    /// Whether the outer shell `occlude()`s mouse hit-testing behind
    /// it (`true` by default — the dropdown / menu surface shape).
    /// Embedders that put an interactive control *behind* the shell
    /// (TextArea's input root) set this to `false` so its own
    /// focus / click / IME handlers stay reachable.
    pub occlude: bool,
}

impl std::fmt::Debug for ScrollViewProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollViewProps")
            .field("id", &self.id)
            .field("scroll_x", &self.scroll_x)
            .field("scroll_y", &self.scroll_y)
            .field("scrollbar_visibility", &self.scrollbar_visibility)
            .field("framed", &self.framed)
            .field("max_content_height", &self.max_content_height)
            .field("gap", &self.gap)
            .field("children", &self.children.len())
            .field("on_scroll", &self.on_scroll.as_ref().map(|_| "<fn>"))
            .field("occlude", &self.occlude)
            .finish()
    }
}

/// Build a headless `ScrollViewProps` for the given `id`, driven by
/// the caller's [`ScrollViewController`].
///
/// The returned props have no content until `.child(...)` /
/// `.children(...)` are called — an empty scroll view renders an
/// empty (non-scrollable) region.
pub fn scroll_view(
    id: impl Into<ElementId>,
    controller: &ScrollViewController,
    _cx: &mut App,
) -> ScrollViewProps {
    ScrollViewProps {
        id: id.into(),
        controller: controller.clone(),
        scroll_x: false,
        scroll_y: true,
        scrollbar_visibility: ScrollbarVisibility::default(),
        framed: false,
        max_content_height: None,
        gap: None,
        children: Vec::new(),
        on_scroll: None,
        occlude: true,
    }
}

impl ScrollViewProps {
    /// Enable or disable horizontal scrolling.
    pub fn scroll_x(mut self, v: bool) -> Self {
        self.scroll_x = v;
        self
    }

    /// Enable or disable vertical scrolling (on by default).
    pub fn scroll_y(mut self, v: bool) -> Self {
        self.scroll_y = v;
        self
    }

    /// Control whether the renderer draws its scrollbar overlay.
    pub fn scrollbar_visibility(mut self, v: ScrollbarVisibility) -> Self {
        self.scrollbar_visibility = v;
        self
    }

    /// Opt in to the renderer's themed frame (bg / border / radius).
    pub fn framed(mut self, v: bool) -> Self {
        self.framed = v;
        self
    }

    /// Cap the content area's height: grow with the children up to
    /// `px`, scroll beyond (the dropdown-menu shape).
    pub fn max_content_height(mut self, px: Pixels) -> Self {
        self.max_content_height = Some(px);
        self
    }

    /// Gap between the content area's direct children.
    pub fn gap(mut self, px: Pixels) -> Self {
        self.gap = Some(px);
        self
    }

    /// Control whether the shell occludes mouse hit-testing behind
    /// it. See [`ScrollViewProps::occlude`].
    pub fn occlude(mut self, v: bool) -> Self {
        self.occlude = v;
        self
    }

    /// Append one child to the scrolled content area. Accepts any
    /// element, mirroring `ParentElement::child` ergonomics.
    pub fn child(mut self, child: impl gpui::IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Append many children to the scrolled content area.
    pub fn children<I: IntoIterator<Item: gpui::IntoElement>>(mut self, children: I) -> Self {
        self.children.extend(
            children
                .into_iter()
                .map(gpui::IntoElement::into_any_element),
        );
        self
    }

    /// Register a callback fired on user-driven scrolls. The
    /// renderer invokes it from its wheel / drag / keyboard
    /// handlers with the post-scroll `(offset, max_offset)`.
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(Point<Pixels>, Size<Pixels>, &mut Window, &mut App),
    {
        self.on_scroll = Some(Arc::new(f));
        self
    }

    /// `apply` for callers that want a custom render path — sets
    /// the id only, matching the virtual-list precedent (the
    /// content lives in the props, so a caller-provided shell
    /// cannot host it).
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render through the registered `ScrollViewRenderer`. The
    /// props are consumed so the children can be transferred into
    /// the renderer's content area.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::ScrollView as ScrollViewMarker;
        use crate::renderer::scroll_view::ScrollViewRenderer;
        let r: &std::sync::Arc<dyn ScrollViewRenderer> = cx
            .renderer_arc::<ScrollViewMarker, dyn ScrollViewRenderer>()
            .expect("ScrollViewRenderer registered");
        r.compose(self, cx)
    }

    /// Render a bare scroll view from raw parts, for
    /// renderer-internal embedding (dropdown menus, text areas).
    /// Bypasses the [`scroll_view`] factory because embedders hold
    /// a `&App`, not `&mut App`; the defaults match the factory
    /// (vertical, scrollbars on, unframed, occluding).
    pub fn compose_embedded(
        id: impl Into<ElementId>,
        controller: &ScrollViewController,
        max_content_height: Option<Pixels>,
        gap: Option<Pixels>,
        children: Vec<gpui::AnyElement>,
        cx: &gpui::App,
    ) -> Stateful<Div> {
        Self::compose_embedded_with_occlude(
            id,
            controller,
            max_content_height,
            gap,
            children,
            true,
            cx,
        )
    }

    /// [`compose_embedded`] with an explicit `occlude` flag. Set it
    /// to `false` when the scroller wraps a control that must keep
    /// its own focus / click / IME hit-testing reachable (the
    /// TextArea input root).
    pub fn compose_embedded_with_occlude(
        id: impl Into<ElementId>,
        controller: &ScrollViewController,
        max_content_height: Option<Pixels>,
        gap: Option<Pixels>,
        children: Vec<gpui::AnyElement>,
        occlude: bool,
        cx: &gpui::App,
    ) -> Stateful<Div> {
        Self {
            id: id.into(),
            controller: controller.clone(),
            scroll_x: false,
            scroll_y: true,
            scrollbar_visibility: ScrollbarVisibility::default(),
            framed: false,
            max_content_height,
            gap,
            children,
            on_scroll: None,
            occlude,
        }
        .render(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    /// The eager-clamp math, exercised on a raw handle via
    /// `set_offset` (the same primitive the controller wraps).
    /// `max_offset` stays zero until a real prepaint runs, which
    /// pins every offset to 0 — proving the clamp keeps unknown
    /// axes locked instead of accumulating drift.
    #[test]
    fn clamp_keeps_unmeasured_axes_locked() {
        let c = ScrollViewController::new();
        c.scroll_by(point(px(0.), px(-100.)));
        assert_eq!(c.offset(), point(px(0.), px(0.)));
        assert!(c.is_at_top());
        assert!(c.is_at_bottom());
        assert!(c.is_at_left());
        assert!(c.is_at_right());
    }

    #[test]
    fn defaults_match_winui_bare_scrollviewer() {
        let c = ScrollViewController::default();
        assert_eq!(c.max_offset(), size(px(0.), px(0.)));
        assert_eq!(c.viewport_size(), size(px(0.), px(0.)));
        assert_eq!(c.content_size(), size(px(0.), px(0.)));
        assert_eq!(ScrollbarVisibility::default(), ScrollbarVisibility::Always);
    }
}
