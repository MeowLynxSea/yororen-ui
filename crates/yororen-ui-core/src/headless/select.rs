//! Headless `select` — option list with a current value and
//! keyboard navigation. The renderer draws a trigger + dropdown.

use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful};

use crate::animation::{AnimatedPresenceState, AnimatedVisibility};
use crate::headless::list_navigable::{ListNavigable, highlight_next, highlight_prev};

#[derive(Clone, Debug)]
pub struct SelectOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl SelectOption {
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

pub type SelectChangeCallback = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct SelectState {
    pub open: bool,
    /// Scroll controller for the dropdown option list (see
    /// `ListboxState::scroll`).
    pub scroll: crate::headless::scroll_view::ScrollViewController,
    pub animation: AnimatedVisibility,
    pub value: Option<SharedString>,
    pub options: Vec<SelectOption>,
    pub highlighted_index: Option<usize>,
    pub placeholder: SharedString,
    pub dismiss_on_escape: bool,
    on_change: Option<SelectChangeCallback>,
}

impl SelectState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            open: false,
            scroll: crate::headless::scroll_view::ScrollViewController::new(),
            animation: AnimatedVisibility::new(),
            value: None,
            options: Vec::new(),
            highlighted_index: None,
            placeholder: "Select an option…".into(),
            dismiss_on_escape: true,
            on_change: None,
        })
    }

    /// Scroll the highlighted item into view through the attached
    /// [`ScrollViewController`](crate::headless::scroll_view::ScrollViewController).
    /// Renderers call this after keyboard highlight changes so the
    /// highlighted option stays visible in long lists. No-op when
    /// nothing is highlighted or the list fits.
    pub fn scroll_highlighted_into_view(&self) {
        if let Some(ix) = self.highlighted_index {
            self.scroll.scroll_to_item(ix);
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.animation.show();
    }
    pub fn close(&mut self) {
        self.open = false;
        self.animation.hide();
    }
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.animation.toggle();
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn is_visible(&self) -> bool {
        self.animation.is_visible()
    }
    pub fn set_options(&mut self, opts: Vec<SelectOption>) {
        self.options = opts;
    }
    pub fn set_value(&mut self, v: impl Into<SharedString>) {
        self.value = Some(v.into());
    }
    pub fn set_placeholder(&mut self, p: impl Into<SharedString>) {
        self.placeholder = p.into();
    }
    pub fn highlight(&mut self, i: usize) {
        if i < self.options.len() {
            self.highlighted_index = Some(i);
        }
    }
    pub fn highlight_next(&mut self) {
        highlight_next(self);
    }
    pub fn highlight_prev(&mut self) {
        highlight_prev(self);
    }
    pub fn set_on_change<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
    }
    pub fn invoke_change(&self, value: SharedString, window: &mut gpui::Window, cx: &mut App) {
        if let Some(f) = &self.on_change {
            f(value, window, cx);
        }
    }
    /// Pick an option. Headless data-layer action — does
    /// NOT belong in the renderer. Writes the new value,
    /// closes the dropdown, and fires the user-supplied
    /// `on_change` callback. Renderer composes visuals +
    /// wires this as the item's click handler.
    pub fn pick(&mut self, value: SharedString, window: &mut gpui::Window, cx: &mut App) {
        self.value = Some(value.clone());
        self.open = false;
        self.animation.hide();
        self.invoke_change(value, window, cx);
    }
    /// Borrow the on_change callback (if any). Used by
    /// renderers to fire the user's pick handler after
    /// they mutate `value` themselves (since `set_value`
    /// alone doesn't fire `on_change`).
    pub fn on_change(&self) -> Option<&SelectChangeCallback> {
        self.on_change.as_ref()
    }
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(opt) = self.options.get(i)
        {
            let value = opt.value.clone();
            self.value = Some(value.clone());
            self.open = false;
            self.animation.hide();
            self.invoke_change(value, window, cx);
        }
    }
}

impl AnimatedPresenceState for SelectState {
    fn visibility(&self) -> &AnimatedVisibility {
        &self.animation
    }
    fn visibility_mut(&mut self) -> &mut AnimatedVisibility {
        &mut self.animation
    }
}

impl ListNavigable for SelectState {
    fn options_len(&self) -> usize {
        self.options.len()
    }
    fn highlighted_index(&self) -> Option<usize> {
        self.highlighted_index
    }
    fn set_highlighted(&mut self, i: usize) {
        self.highlighted_index = Some(i);
    }
}

#[derive(Clone)]
pub struct SelectProps {
    pub id: ElementId,
    pub state: Entity<SelectState>,
}

pub fn select(id: impl Into<ElementId>, state: Entity<SelectState>) -> SelectProps {
    SelectProps {
        id: id.into(),
        state,
    }
}

impl SelectProps {
    /// Apply the headless contract to the renderer-built `el`.
    /// Sets the element id only. The renderer is responsible
    /// for visuals AND for wiring the click handler that
    /// toggles the dropdown — the renderer composes the
    /// full UI (trigger + dropdown) and decides how to
    /// route the click to avoid bubbling from option items
    /// back to the trigger.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the select trigger using the registered
    /// `SelectRenderer`. Returns a `Stateful<Div>` with the
    /// element id. The renderer decides bg / border / padding
    /// based on the `state` entity.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::Select as SelectMarker;
        use crate::renderer::select::SelectRenderer;

        let r: &Arc<dyn SelectRenderer> = cx
            .renderer_arc::<SelectMarker, dyn SelectRenderer>()
            .expect("SelectRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
