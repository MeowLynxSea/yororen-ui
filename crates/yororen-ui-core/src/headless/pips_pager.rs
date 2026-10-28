//! Headless `PipsPager` — a page-indicator control (WinUI
//! `PipsPager`): a row (or column) of dots, one per page, with the
//! current page's dot highlighted. Optional `…` ellipsis slots elide
//! long page counts down to a moving window around the current
//! page, and optional previous/next chevron buttons flank the pips.
//!
//! State lives in an [`Entity<PipsPagerState>`] owned by the caller
//! (the same shape as [`crate::headless::grid_view`]'s
//! `GridViewState`): the renderer reads `current_page` /
//! `orientation` / `max_visible_pips` / `show_nav_buttons` from the
//! entity and wires pip clicks, chevron clicks, and ← / → / Home /
//! End keystrokes back into the state's navigation methods.

use std::sync::Arc;

use gpui::{
    App, AppContext, Div, ElementId, Entity, FocusHandle, Focusable, InteractiveElement, Stateful,
};

/// Direction the pips are laid out in. `Horizontal` (default) draws
/// a row of dots flanked left/right by the nav buttons; `Vertical`
/// draws a column flanked top/bottom.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum PipsPagerOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// One rendered slot in the elided pip strip. `Page(i)` is a real,
/// clickable dot for page `i`; the ellipsis variants are the `…`
/// indicators that stand for the elided run before / after the
/// visible window — clicking one jumps a whole window in that
/// direction (WinUI behaviour).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PipSlot {
    Page(usize),
    EllipsisBefore,
    EllipsisAfter,
}

pub type PipsPagerChangeCallback = Arc<dyn Fn(usize, &mut gpui::Window, &mut App)>;

/// Default page count for a fresh state. Purely a seed — call
/// [`PipsPagerState::set_page_count`] to change it; the pip strip
/// and the navigation methods both follow the state.
pub const DEFAULT_PAGE_COUNT: usize = 5;

/// Default elided-window width (page pips visible at once) when
/// `max_visible_pips` is set without an explicit width.
pub const DEFAULT_MAX_VISIBLE_PIPS: usize = 5;

#[derive(Clone)]
pub struct PipsPagerState {
    pub page_count: usize,
    pub current_page: usize,
    pub orientation: PipsPagerOrientation,
    /// When `Some(n)` and `page_count > n`, only a moving window of
    /// `n` page pips renders, elided with `…` slots on either side.
    /// `None` renders every pip. The window counts page pips only —
    /// ellipsis slots are extra.
    pub max_visible_pips: Option<usize>,
    /// Whether the renderer draws the previous/next chevron buttons
    /// flanking the pip strip.
    pub show_nav_buttons: bool,
    /// Focus handle minted in `new`. The renderer wires
    /// `track_focus` + `on_key_down` against this handle so ← / →
    /// move pages; the handle is private so the public `Focusable`
    /// trait impl owns the consumer-facing name.
    focus_handle: FocusHandle,
    on_change: Option<PipsPagerChangeCallback>,
}

impl PipsPagerState {
    pub fn new(app: &mut App) -> Entity<Self> {
        let focus_handle = app.focus_handle();
        app.new(|_| Self {
            page_count: DEFAULT_PAGE_COUNT,
            current_page: 0,
            orientation: PipsPagerOrientation::default(),
            max_visible_pips: None,
            show_nav_buttons: false,
            focus_handle,
            on_change: None,
        })
    }

    /// Public focus handle accessor. Mirrors the `Focusable`
    /// impl below; duplicated so the renderer can grab the handle
    /// without going through the trait object.
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    /// Total number of pages. Setting a smaller count clamps
    /// `current_page` back into range.
    pub fn set_page_count(&mut self, n: usize) {
        self.page_count = n;
        if self.current_page >= n {
            self.current_page = n.saturating_sub(1);
        }
    }

    pub fn set_orientation(&mut self, o: PipsPagerOrientation) {
        self.orientation = o;
    }

    /// Set the elided-window width. Values below 1 are clamped to 1
    /// (a single moving pip).
    pub fn set_max_visible_pips(&mut self, n: Option<usize>) {
        self.max_visible_pips = n.map(|n| n.max(1));
    }

    pub fn set_nav_buttons(&mut self, v: bool) {
        self.show_nav_buttons = v;
    }

    pub fn set_on_change<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(usize, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
    }

    /// Next page. No wrap — WinUI's pager disables past the end
    /// rather than cycling. No-op (no `on_change`) when already on
    /// the last page or when there are no pages.
    pub fn next(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if self.page_count > 0 && self.current_page + 1 < self.page_count {
            self.current_page += 1;
            self.fire_on_change(window, cx);
        }
    }

    /// Previous page. No wrap; no-op on the first page.
    pub fn prev(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.fire_on_change(window, cx);
        }
    }

    /// Jump to a specific page, clamped into `[0, page_count - 1]`.
    /// Fires `on_change` only when the page actually changes.
    pub fn go_to(&mut self, page: usize, window: &mut gpui::Window, cx: &mut App) {
        if self.page_count == 0 {
            return;
        }
        let target = page.min(self.page_count - 1);
        if target != self.current_page {
            self.current_page = target;
            self.fire_on_change(window, cx);
        }
    }

    /// Jump one elided window forward (the `…` after the window).
    /// Falls back to a single step when no window is configured.
    pub fn skip_forward(&mut self, window: &mut gpui::Window, cx: &mut App) {
        let step = self.window_width().max(1);
        if self.page_count > 0 && self.current_page + 1 < self.page_count {
            let target = (self.current_page + step).min(self.page_count - 1);
            self.current_page = target;
            self.fire_on_change(window, cx);
        }
    }

    /// Jump one elided window backward (the `…` before the window).
    pub fn skip_backward(&mut self, window: &mut gpui::Window, cx: &mut App) {
        let step = self.window_width().max(1);
        if self.current_page > 0 {
            let target = self.current_page.saturating_sub(step);
            self.current_page = target;
            self.fire_on_change(window, cx);
        }
    }

    /// The elided-window width actually in effect: `None` when
    /// every pip renders.
    fn window_width(&self) -> usize {
        match self.max_visible_pips {
            Some(n) if self.page_count > n => n,
            _ => usize::MAX,
        }
    }

    /// The slots the renderer should draw, derived from the current
    /// state. Pure function of `(current_page, page_count,
    /// max_visible_pips)` — see [`visible_pip_window`].
    pub fn pip_slots(&self) -> Vec<PipSlot> {
        visible_pip_window(self.current_page, self.page_count, self.max_visible_pips)
    }

    /// Whether the leading `…` slot actually elides a non-empty run
    /// (i.e. the visible window starts past page 0). When `false`
    /// the slot is rendered only to keep the control's width
    /// constant and must not react to clicks.
    pub fn ellipsis_before_elides(&self) -> bool {
        pip_window_start(self.current_page, self.page_count, self.max_visible_pips)
            .is_some_and(|s| s > 0)
    }

    /// Whether the trailing `…` slot actually elides a non-empty run
    /// (i.e. the visible window ends before the last page).
    pub fn ellipsis_after_elides(&self) -> bool {
        pip_window_end(self.current_page, self.page_count, self.max_visible_pips)
            .is_some_and(|e| e < self.page_count)
    }

    fn fire_on_change(&self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(f) = &self.on_change {
            f(self.current_page, window, cx);
        }
    }
}

/// Compute the rendered pip slots: every page when no elision is
/// configured (or the count fits), otherwise a window of
/// `max_visible` page pips centred on `current`.
///
/// When the page range is elided, **both** ellipsis slots are always
/// present — `EllipsisBefore` before the first visible pip and
/// `EllipsisAfter` after the last. At the very start/end of the page
/// range the outer slot stands for an empty run and is inert to
/// clicks, but it is still rendered so the control's width stays
/// constant as the window slides (this is how WinUI's `PipsPager`
/// avoids reflowing while you page). The window counts page pips
/// only — ellipsis slots are extra.
///
/// The window is centred (not leading-edge) so the current pip
/// stays in the middle of the strip while stepping page by page —
/// the window slides under it, and only clamps at the very
/// beginning/end of the page range, exactly like WinUI.
pub fn visible_pip_window(
    current: usize,
    count: usize,
    max_visible: Option<usize>,
) -> Vec<PipSlot> {
    if count == 0 {
        return Vec::new();
    }
    let max_visible = max_visible.unwrap_or(count).max(1);
    if count <= max_visible {
        return (0..count).map(PipSlot::Page).collect();
    }

    // Centre the window on `current`, clamped so it stays inside
    // [0, count - max_visible].
    let half = max_visible / 2;
    let start = current.saturating_sub(half).min(count - max_visible);
    let end = start + max_visible;

    let mut slots = Vec::with_capacity(max_visible + 2);
    slots.push(PipSlot::EllipsisBefore);
    slots.extend((start..end).map(PipSlot::Page));
    slots.push(PipSlot::EllipsisAfter);
    slots
}

/// The start index of the centred, clamped page window actually in
/// effect (`None` when the count fits without elision).
pub fn pip_window_start(current: usize, count: usize, max_visible: Option<usize>) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let max_visible = max_visible.unwrap_or(count).max(1);
    if count <= max_visible {
        return None;
    }
    let half = max_visible / 2;
    Some(current.saturating_sub(half).min(count - max_visible))
}

/// Exclusive end index of the page window (`None` when no elision).
pub fn pip_window_end(current: usize, count: usize, max_visible: Option<usize>) -> Option<usize> {
    pip_window_start(current, count, max_visible).map(|s| {
        let max_visible = max_visible.unwrap_or(count).max(1);
        s + max_visible
    })
}

/// `Focusable` impl so the platform's focus traversal (Tab key)
/// and the renderer's `track_focus` can find the pager's handle —
/// without it, arrow-key nav never fires.
impl Focusable for PipsPagerState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[derive(Clone)]
pub struct PipsPagerProps {
    pub id: ElementId,
    pub state: Entity<PipsPagerState>,
}

pub fn pips_pager(id: impl Into<ElementId>, state: Entity<PipsPagerState>) -> PipsPagerProps {
    PipsPagerProps {
        id: id.into(),
        state,
    }
}

impl PipsPagerProps {
    /// Apply the headless contract to the renderer-built `el`.
    /// Sets the element id only. The renderer is responsible for
    /// visuals AND for wiring each pip's click handler to
    /// `state.go_to(page, …)`.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the pager via the registered `PipsPagerRenderer`.
    /// Returns a `Stateful<Div>` containing the pip strip (and the
    /// nav buttons when enabled) with the current page's pip
    /// highlighted; the registered renderer decides visual
    /// treatment (dot size, spacing, chevron glyphs, etc.).
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::PipsPager as PipsPagerMarker;
        use crate::renderer::pips_pager::PipsPagerRenderer;

        let r: &Arc<dyn PipsPagerRenderer> = cx
            .renderer_arc::<PipsPagerMarker, dyn PipsPagerRenderer>()
            .expect("PipsPagerRenderer registered");
        r.compose(&self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_elision_shows_every_page() {
        let slots = visible_pip_window(2, 5, None);
        assert_eq!(
            slots,
            vec![
                PipSlot::Page(0),
                PipSlot::Page(1),
                PipSlot::Page(2),
                PipSlot::Page(3),
                PipSlot::Page(4),
            ]
        );
    }

    #[test]
    fn count_fitting_window_shows_every_page() {
        let slots = visible_pip_window(1, 5, Some(5));
        assert_eq!(slots.len(), 5);
        assert!(!slots.contains(&PipSlot::EllipsisBefore));
    }

    #[test]
    fn early_pages_keep_both_ellipsis_slots_for_constant_width() {
        // The leading slot stands for an empty run but is still
        // rendered, so the control does not reflow while paging.
        let slots = visible_pip_window(0, 10, Some(3));
        assert_eq!(
            slots,
            vec![
                PipSlot::EllipsisBefore,
                PipSlot::Page(0),
                PipSlot::Page(1),
                PipSlot::Page(2),
                PipSlot::EllipsisAfter,
            ]
        );
    }

    #[test]
    fn late_pages_keep_both_ellipsis_slots_for_constant_width() {
        let slots = visible_pip_window(9, 10, Some(3));
        assert_eq!(
            slots,
            vec![
                PipSlot::EllipsisBefore,
                PipSlot::Page(7),
                PipSlot::Page(8),
                PipSlot::Page(9),
                PipSlot::EllipsisAfter,
            ]
        );
    }

    #[test]
    fn middle_pages_show_both_ellipses_and_stay_centred() {
        // Window of 3 centred on 4 → 3..6, elided on both sides.
        let slots = visible_pip_window(4, 10, Some(3));
        assert_eq!(
            slots,
            vec![
                PipSlot::EllipsisBefore,
                PipSlot::Page(3),
                PipSlot::Page(4),
                PipSlot::Page(5),
                PipSlot::EllipsisAfter,
            ]
        );
        // Stepping to 5 slides the window so the current pip stays
        // in the centre slot of the strip.
        let slots = visible_pip_window(5, 10, Some(3));
        assert_eq!(
            slots,
            vec![
                PipSlot::EllipsisBefore,
                PipSlot::Page(4),
                PipSlot::Page(5),
                PipSlot::Page(6),
                PipSlot::EllipsisAfter,
            ]
        );
    }

    #[test]
    fn even_window_biases_towards_start() {
        // Window of 4 centred on 0 → start = 0 (clamped), 0..4.
        // Both ellipsis slots render even at the start of the range.
        let slots = visible_pip_window(0, 9, Some(4));
        assert_eq!(
            slots,
            vec![
                PipSlot::EllipsisBefore,
                PipSlot::Page(0),
                PipSlot::Page(1),
                PipSlot::Page(2),
                PipSlot::Page(3),
                PipSlot::EllipsisAfter,
            ]
        );
        // Centred on 5 → half = 2, start = 3, so 3..7.
        let slots = visible_pip_window(5, 9, Some(4));
        assert_eq!(slots[0], PipSlot::EllipsisBefore);
        assert_eq!(slots[1], PipSlot::Page(3));
        assert_eq!(*slots.last().unwrap(), PipSlot::EllipsisAfter);
    }

    #[test]
    fn zero_pages_render_nothing() {
        assert!(visible_pip_window(0, 0, Some(3)).is_empty());
    }

    #[test]
    fn max_visible_below_one_is_clamped() {
        let slots = visible_pip_window(0, 3, Some(0));
        assert_eq!(
            slots,
            vec![
                PipSlot::EllipsisBefore,
                PipSlot::Page(0),
                PipSlot::EllipsisAfter,
            ]
        );
    }
}
