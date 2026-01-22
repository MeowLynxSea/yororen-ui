use gpui::{
    AnyElement, IntoElement, ListAlignment, ListSizingBehavior, Pixels, RenderOnce, Styled, list,
};

/// Controller for a [`VirtualList`].
///
/// This is intentionally a thin wrapper over `gpui::ListState` so Yororen UI users
/// don't have to call `reset/splice/scroll_to_reveal_item` directly.
#[derive(Clone, Debug)]
pub struct VirtualListController {
    state: gpui::ListState,
}

impl VirtualListController {
    pub fn new(state: gpui::ListState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> gpui::ListState {
        self.state.clone()
    }

    pub fn reset(&self, element_count: usize) {
        self.state.reset(element_count);
    }

    pub fn splice(&self, old_range: std::ops::Range<usize>, count: usize) {
        self.state.splice(old_range, count);
    }

    pub fn scroll_to_reveal_item(&self, ix: usize) {
        self.state.scroll_to_reveal_item(ix);
    }
}

/// Widget: a virtualized list based on `gpui::list`.
///
/// Yororen UI users should render each item using [`crate::component::virtual_row`]
/// which:
/// - enforces stable keys (prevents state bleed when virtualized rows are recycled)
/// - owns row spacing/dividers (prevents incorrect height inference)
///
/// State ownership:
/// - The underlying `gpui::ListState` must be held by the caller's view/state.
/// - When row heights change (disclosure toggle, async content), notify via
///   [`VirtualListController::splice`] or [`VirtualListController::reset`].
#[derive(IntoElement)]
pub struct VirtualList {
    state: gpui::ListState,
    sizing_behavior: ListSizingBehavior,
    render_row: Box<dyn FnMut(usize, &mut gpui::Window, &mut gpui::App) -> AnyElement + 'static>,
    style: gpui::StyleRefinement,
}

impl VirtualList {
    pub fn new(
        state: gpui::ListState,
        render_row: impl FnMut(usize, &mut gpui::Window, &mut gpui::App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            state,
            sizing_behavior: ListSizingBehavior::default(),
            render_row: Box::new(render_row),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn with_sizing_behavior(mut self, behavior: ListSizingBehavior) -> Self {
        self.sizing_behavior = behavior;
        self
    }
}

impl Styled for VirtualList {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for VirtualList {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        // We must preserve styling that callers applied to `VirtualList`.
        // `gpui::List` is `Styled`, so we can transfer our style refinement onto it.
        let mut inner =
            list(self.state.clone(), self.render_row).with_sizing_behavior(self.sizing_behavior);
        *inner.style() = self.style;

        inner
    }
}

/// An ergonomic container that owns both `gpui::ListState` and a [`VirtualListController`].
///
/// This makes it easy for a view to hold one field and pass `state()` into `virtual_list`,
/// while still having a controller handle for `reset/splice/scroll_to_reveal_item`.
///
/// Note: even when using this handle, the ownership is still at the view level:
/// keep `VirtualListHandle` as a field on your view, not as ephemeral render-local state.
#[derive(Clone, Debug)]
pub struct VirtualListHandle {
    state: gpui::ListState,
    controller: VirtualListController,
}

impl VirtualListHandle {
    pub fn new(item_count: usize, alignment: ListAlignment, overdraw: Pixels) -> Self {
        let state = virtual_list_state(item_count, alignment, overdraw);
        let controller = VirtualListController::new(state.clone());
        Self { state, controller }
    }

    pub fn state(&self) -> gpui::ListState {
        self.state.clone()
    }

    pub fn controller(&self) -> VirtualListController {
        self.controller.clone()
    }
}

/// Construct a new virtual list widget.
#[track_caller]
pub fn virtual_list(
    state: gpui::ListState,
    render_row: impl FnMut(usize, &mut gpui::Window, &mut gpui::App) -> AnyElement + 'static,
) -> VirtualList {
    VirtualList::new(state, render_row)
}

/// Construct list state for a virtual list.
pub fn virtual_list_state(
    item_count: usize,
    alignment: ListAlignment,
    overdraw: Pixels,
) -> gpui::ListState {
    gpui::ListState::new(item_count, alignment, overdraw)
}
