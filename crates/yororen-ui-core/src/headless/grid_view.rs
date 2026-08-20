//! Headless `GridView` — a single-select grid of uniformly sized
//! tiles. Unlike [`crate::headless::listbox`], which lays options
//! out one per row, the grid view chunks its options into rows of
//! `columns` tiles; ← / → walk the linear option order (wrapping,
//! exactly like `listbox`) while ↑ / ↓ jump by a whole row stride
//! (no vertical wrap — you cannot step above the first row or
//! below the last).

use std::sync::Arc;

use gpui::{
    App, AppContext, Div, ElementId, Entity, FocusHandle, Focusable, InteractiveElement,
    SharedString, Stateful,
};

use super::list_navigable::{ListNavigable, highlight_next, highlight_prev};

#[derive(Clone, Debug)]
pub struct GridViewOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl GridViewOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}

pub type GridViewChangeCallback = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

/// Default column count for a fresh state. Purely a seed — call
/// [`GridViewState::set_columns`] to change it; the keyboard-nav
/// stride and the rendered layout both follow the state.
pub const DEFAULT_COLUMNS: usize = 4;

#[derive(Clone)]
pub struct GridViewState {
    pub options: Vec<GridViewOption>,
    pub highlighted_index: Option<usize>,
    pub selected_value: Option<SharedString>,
    /// Number of tiles per row. Drives both the rendered layout
    /// (renderers chunk options into rows of this many) and the
    /// ↑ / ↓ keyboard-nav stride. Clamped to at least 1 when set.
    pub columns: usize,
    /// Focus handle minted in `new`. The renderer wires
    /// `track_focus` + `on_key_down` against this handle so
    /// arrow keys move highlight, `Enter` selects. The handle
    /// is private so the public `Focusable` trait impl owns
    /// the consumer-facing name.
    focus_handle: FocusHandle,
    on_change: Option<GridViewChangeCallback>,
}

impl GridViewState {
    pub fn new(app: &mut App) -> Entity<Self> {
        let focus_handle = app.focus_handle();
        app.new(|_| Self {
            options: Vec::new(),
            highlighted_index: None,
            selected_value: None,
            columns: DEFAULT_COLUMNS,
            focus_handle,
            on_change: None,
        })
    }

    /// Public focus handle accessor. Mirrors the
    /// `Focusable::focus_handle` impl below; duplicated here so
    /// the renderer can grab the handle without going through
    /// the `Focusable` trait object (which is what `track_focus`
    /// and `is_focused(window)` want).
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub fn set_options(&mut self, opts: Vec<GridViewOption>) {
        self.options = opts;
    }
    pub fn set_selected(&mut self, v: impl Into<SharedString>) {
        self.selected_value = Some(v.into());
    }
    /// Set the number of tiles per row. Values below 1 are
    /// clamped to 1 (a one-column grid degrades to a list).
    pub fn set_columns(&mut self, columns: usize) {
        self.columns = columns.max(1);
    }
    /// →. Same shared wrap-around algorithm as `listbox`, so
    /// ← / → in a grid behave exactly like ↑ / ↓ in a list.
    pub fn highlight_next(&mut self) {
        highlight_next(self);
    }
    /// ←. Same shared wrap-around algorithm as `listbox`.
    pub fn highlight_prev(&mut self) {
        highlight_prev(self);
    }
    /// ↓. Steps down one full row (a whole `columns` stride);
    /// no wrap past the last row. Exposed as a method so the
    /// renderer's key handler stays a one-liner; the testable
    /// free function below carries the actual algorithm.
    pub fn highlight_down(&mut self) {
        highlight_down(self, self.columns);
    }
    /// ↑. Steps up one full row; no wrap past the first row.
    pub fn highlight_up(&mut self) {
        highlight_up(self, self.columns);
    }
    pub fn set_on_change<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
    }
    /// Pick the currently highlighted tile. Headless data-layer
    /// action — does not belong in the renderer. Writes the
    /// selected value and fires the user-supplied `on_change`
    /// callback. The renderer composes visuals + wires this as
    /// each tile's click / Enter handler.
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(opt) = self.options.get(i)
            && !opt.disabled
        {
            let v = opt.value.clone();
            self.selected_value = Some(v.clone());
            if let Some(f) = &self.on_change {
                f(v, window, cx);
            }
        }
    }
    /// Pick a specific tile by value. Headless data-layer
    /// action — does not belong in the renderer. Identical
    /// side-effect to `select_highlighted` but takes the value
    /// directly, so a click handler that already knows which
    /// tile was hit can fire `on_change` without having to first
    /// mutate `highlighted_index`.
    pub fn pick(&mut self, value: SharedString, window: &mut gpui::Window, cx: &mut App) {
        if self.options.iter().any(|o| o.value == value) {
            self.selected_value = Some(value.clone());
            if let Some(f) = &self.on_change {
                f(value, window, cx);
            }
        }
    }
}

impl ListNavigable for GridViewState {
    fn options_len(&self) -> usize {
        self.options.len()
    }
    fn highlighted_index(&self) -> Option<usize> {
        self.highlighted_index
    }
    fn set_highlighted(&mut self, i: usize) {
        self.highlighted_index = Some(i);
    }
    /// Disabled tiles are visible but not selectable. The
    /// shared `highlight_next` / `highlight_prev` will skip over
    /// them rather than land on them.
    fn is_selectable(&self, i: usize) -> bool {
        self.options.get(i).map(|o| !o.disabled).unwrap_or(false)
    }
}

/// ↓ for any [`ListNavigable`] state laid out in a grid. Steps a
/// whole `columns` stride down; no wrap past the last row. When
/// the tile directly below is disabled, drifts right to the next
/// selectable tile (the user lands somewhere in the target row
/// rather than the key silently doing nothing).
pub fn highlight_down<N: ListNavigable>(state: &mut N, columns: usize) {
    let len = state.options_len();
    if len == 0 {
        return;
    }
    let stride = columns.max(1);
    let candidate = match state.highlighted_index() {
        None => 0,
        Some(i) => {
            let c = i + stride;
            if c >= len {
                return;
            }
            c
        }
    };
    if state.is_selectable(candidate) {
        state.set_highlighted(candidate);
    } else {
        for c in (candidate + 1)..len {
            if state.is_selectable(c) {
                state.set_highlighted(c);
                return;
            }
        }
    }
}

/// ↑ for any [`ListNavigable`] state laid out in a grid. Steps a
/// whole `columns` stride up; no wrap past the first row. When
/// the tile directly above is disabled, drifts left to the next
/// selectable tile.
pub fn highlight_up<N: ListNavigable>(state: &mut N, columns: usize) {
    let len = state.options_len();
    if len == 0 {
        return;
    }
    let stride = columns.max(1);
    let candidate = match state.highlighted_index() {
        None => 0,
        Some(i) => {
            if i < stride {
                return;
            }
            i - stride
        }
    };
    if state.is_selectable(candidate) {
        state.set_highlighted(candidate);
    } else {
        for c in (0..candidate).rev() {
            if state.is_selectable(c) {
                state.set_highlighted(c);
                return;
            }
        }
    }
}

/// `Focusable` impl so the platform's focus traversal (Tab key)
/// and the renderer's `track_focus` can find the grid view's
/// handle. Without this, `Window::handle_input(&focus_handle,
/// …)` can't deliver key events to the grid and arrow-key
/// nav never fires.
impl Focusable for GridViewState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[derive(Clone)]
pub struct GridViewProps {
    pub id: ElementId,
    pub state: Entity<GridViewState>,
}

pub fn grid_view(id: impl Into<ElementId>, state: Entity<GridViewState>) -> GridViewProps {
    GridViewProps {
        id: id.into(),
        state,
    }
}

impl GridViewProps {
    /// Apply the headless contract to the renderer-built `el`.
    /// Sets the element id only. The renderer is responsible
    /// for visuals AND for wiring each tile's click handler to
    /// `state.pick(value, …)` (or to mutate `highlighted_index`
    /// and then call `state.select_highlighted(…)`).
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the grid view via the registered
    /// `GridViewRenderer`. Returns a `Stateful<Div>` containing
    /// one tile per option chunked into rows of `columns`, with
    /// the highlighted tile styled accordingly and the selected
    /// tile marked. Clicking a tile fires `state.pick(value, …)`;
    /// the registered renderer decides visual treatment
    /// (backgrounds, hover, dividers, etc.).
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::grid_view::GridViewRenderer;
        use crate::renderer::markers::GridView as GridViewMarker;

        let r: &Arc<dyn GridViewRenderer> = cx
            .renderer_arc::<GridViewMarker, dyn GridViewRenderer>()
            .expect("GridViewRenderer registered");
        r.compose(&self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal in-test state used to assert the grid stride
    /// algorithm without dragging in the gpui entity machinery.
    /// Mirrors the shape of `GridViewState`.
    #[derive(Default)]
    struct GridState {
        options: Vec<String>,
        highlighted: Option<usize>,
    }

    impl GridState {
        fn with(options: &[&str], highlighted: Option<usize>) -> Self {
            Self {
                options: options.iter().map(|s| s.to_string()).collect(),
                highlighted,
            }
        }
    }

    impl ListNavigable for GridState {
        fn options_len(&self) -> usize {
            self.options.len()
        }
        fn highlighted_index(&self) -> Option<usize> {
            self.highlighted
        }
        fn set_highlighted(&mut self, i: usize) {
            self.highlighted = Some(i);
        }
    }

    #[test]
    fn down_steps_one_row_stride() {
        let mut s = GridState::with(&["a", "b", "c", "d", "e", "f"], Some(0));
        highlight_down(&mut s, 3);
        assert_eq!(s.highlighted, Some(3));
        highlight_down(&mut s, 3);
        assert_eq!(s.highlighted, Some(3), "no wrap past the last row");
    }

    #[test]
    fn up_steps_one_row_stride() {
        let mut s = GridState::with(&["a", "b", "c", "d", "e", "f"], Some(4));
        highlight_up(&mut s, 3);
        assert_eq!(s.highlighted, Some(1));
        highlight_up(&mut s, 3);
        assert_eq!(s.highlighted, Some(1), "no wrap past the first row");
    }

    #[test]
    fn down_from_none_starts_at_zero() {
        let mut s = GridState::with(&["a", "b"], None);
        highlight_down(&mut s, 2);
        assert_eq!(s.highlighted, Some(0));
    }

    #[test]
    fn up_from_none_starts_at_zero() {
        let mut s = GridState::with(&["a", "b"], None);
        highlight_up(&mut s, 2);
        assert_eq!(s.highlighted, Some(0));
    }

    #[test]
    fn empty_state_is_a_noop() {
        let mut s = GridState::default();
        highlight_down(&mut s, 3);
        highlight_up(&mut s, 3);
        assert_eq!(s.highlighted, None);
    }

    #[test]
    fn short_last_row_clamps_instead_of_overflowing() {
        // 5 options, 3 columns: index 3 and 4 form a short row.
        let mut s = GridState::with(&["a", "b", "c", "d", "e"], Some(3));
        highlight_down(&mut s, 3);
        assert_eq!(s.highlighted, Some(3), "3 + 3 overflows the 5 options");
    }

    /// Only odd indices are selectable, so tiles directly
    /// below/above the highlighted one may be disabled.
    struct SkipOddState {
        highlighted: Option<usize>,
    }

    impl ListNavigable for SkipOddState {
        fn options_len(&self) -> usize {
            4
        }
        fn highlighted_index(&self) -> Option<usize> {
            self.highlighted
        }
        fn set_highlighted(&mut self, i: usize) {
            self.highlighted = Some(i);
        }
        fn is_selectable(&self, i: usize) -> bool {
            i % 2 == 1
        }
    }

    #[test]
    fn down_drifts_right_over_disabled_tile() {
        // Highlighted at 0, tile directly below (2) is disabled;
        // the scan drifts right to 3.
        let mut s = SkipOddState { highlighted: Some(0) };
        highlight_down(&mut s, 2);
        assert_eq!(s.highlighted, Some(3));
    }

    #[test]
    fn up_drifts_left_over_disabled_tile() {
        // Highlighted at 3, tile directly above (1) is selectable.
        let mut s = SkipOddState { highlighted: Some(3) };
        highlight_up(&mut s, 2);
        assert_eq!(s.highlighted, Some(1));
    }
}
