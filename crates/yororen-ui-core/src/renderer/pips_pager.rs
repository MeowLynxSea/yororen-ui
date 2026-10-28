//! `PipsPagerRenderer` — visual contract for `PipsPager`.
//!
//! The renderer reads everything from the props' `Entity<
//! PipsPagerState>` (page count, current page, orientation,
//! elision window, nav buttons) and wires pip clicks and ← / → /
//! Home / End keystrokes back into the state's navigation methods
//! (`go_to` / `next` / `prev`). The elided slot strip comes from
//! [`PipsPagerState::pip_slots`] — the renderer never re-derives
//! the windowing.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::pips_pager::{PipsPagerOrientation, PipsPagerProps};

/// Pure data view of a pager, derived from the entity state at
/// render time. Renderer helpers consume this to look up theme
/// tokens without holding the entity borrow.
#[derive(Clone, Copy, Debug)]
pub struct PipsPagerRenderState {
    pub page_count: usize,
    pub current_page: usize,
    pub orientation: PipsPagerOrientation,
    pub max_visible_pips: Option<usize>,
    pub show_nav_buttons: bool,
    /// Whether the leading `…` slot actually elides a non-empty run
    /// (it is otherwise rendered at fixed spot, inert, to keep the
    /// control width constant while the window slides).
    pub ellipsis_before_elides: bool,
    /// Same for the trailing `…` slot.
    pub ellipsis_after_elides: bool,
}

pub trait PipsPagerRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` pager — the pip strip (and
    /// the nav buttons when enabled) with the current page's pip
    /// highlighted. Clicking a pip fires `state.go_to(page, …)`;
    /// the registered renderer decides visual treatment (dot size,
    /// spacing, chevron glyphs, etc.).
    fn compose(&self, props: &PipsPagerProps, cx: &App) -> Stateful<Div>;
}
