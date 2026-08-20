//! `GridViewRenderer` — visual contract for `GridView`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (bg / border / hover_bg / selected_bg / fg / padding /
//! min_item_size / border_radius) stay on the concrete renderer
//! type. The headless layer owns the option list, highlight
//! index, selected value, and column count; the renderer decides
//! how those are painted into a `Stateful<Div>` containing one
//! tile per option chunked into rows.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::grid_view::GridViewProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct GridViewRenderState {
    pub item_count: usize,
    pub columns: usize,
}

pub trait GridViewRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the grid view. The
    /// renderer iterates `props.state.options` and emits one
    /// tile per option, chunked into rows of the state's
    /// `columns`, applying the highlight / selected / hover
    /// styling of its choice. Click handlers are wired to
    /// `state.pick(value, …)`; the headless layer fires
    /// `on_change` on the user's behalf.
    fn compose(&self, props: &GridViewProps, cx: &App) -> Stateful<Div>;
}
