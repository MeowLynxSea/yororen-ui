//! Headless `dropdown_menu` — a vertical list of items triggered
//! by a button.

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful,
};

use crate::animation::{AnimatedPresenceState, AnimatedVisibility};

#[derive(Clone, Debug)]
pub enum DropdownItem {
    Item(DropdownMenuItem),
    Separator,
    Group(DropdownMenuGroup),
}

#[derive(Clone, Debug)]
pub struct DropdownMenuItem {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub disabled: bool,
    pub shortcut: Option<Vec<String>>,
}

impl DropdownMenuItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            shortcut: None,
        }
    }
    pub fn icon(mut self, i: impl Into<SharedString>) -> Self {
        self.icon = Some(i.into());
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn shortcut(mut self, k: Vec<String>) -> Self {
        self.shortcut = Some(k);
        self
    }
}

#[derive(Clone, Debug)]
pub struct DropdownMenuGroup {
    pub label: SharedString,
    pub items: Vec<DropdownMenuItem>,
}

pub type DropdownSelectCallback = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct DropdownMenuState {
    pub open: bool,
    /// Scroll controller for the menu list (see
    /// `ListboxState::scroll`).
    pub scroll: crate::headless::scroll_view::ScrollViewController,
    pub animation: AnimatedVisibility,
    pub highlighted_index: Option<usize>,
    pub dismiss_on_escape: bool,
    /// When `true` (the default), clicking anywhere outside the
    /// trigger + menu area closes the menu. Renderers wire this
    /// up via `gpui::on_mouse_down_out` on the owning element.
    pub dismiss_on_outside_click: bool,
    pub items: Vec<DropdownItem>,
    on_select: Option<DropdownSelectCallback>,
}

impl DropdownMenuState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            open: false,
            scroll: crate::headless::scroll_view::ScrollViewController::new(),
            animation: AnimatedVisibility::new(),
            highlighted_index: None,
            dismiss_on_escape: true,
            dismiss_on_outside_click: true,
            items: Vec::new(),
            on_select: None,
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
    pub fn set_items(&mut self, items: Vec<DropdownItem>) {
        self.items = items;
    }
    pub fn set_dismiss_on_outside_click(&mut self, v: bool) {
        self.dismiss_on_outside_click = v;
    }
    pub fn highlight_next(&mut self) {
        // Skip separators.
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let mut i = match self.highlighted_index {
            Some(i) => i + 1,
            None => 0,
        };
        while i < len && matches!(self.items[i], DropdownItem::Separator) {
            i += 1;
        }
        if i < len {
            self.highlighted_index = Some(i);
        }
    }
    pub fn highlight_prev(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let mut i = match self.highlighted_index {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        while i > 0 && matches!(self.items[i], DropdownItem::Separator) {
            i -= 1;
        }
        self.highlighted_index = Some(i);
    }
    pub fn set_on_select<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_select = Some(Arc::new(f));
    }
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(DropdownItem::Item(it)) = self.items.get(i)
        {
            let id = it.id.clone();
            self.open = false;
            self.animation.hide();
            if let Some(f) = &self.on_select {
                f(id, window, cx);
            }
        }
    }
}

impl AnimatedPresenceState for DropdownMenuState {
    fn visibility(&self) -> &AnimatedVisibility {
        &self.animation
    }
    fn visibility_mut(&mut self) -> &mut AnimatedVisibility {
        &mut self.animation
    }
}

pub struct DropdownMenuProps {
    pub id: ElementId,
    pub state: Entity<DropdownMenuState>,
    /// Caller-supplied trigger element. Rendered in normal
    /// flow; the caller's click handler is expected to call
    /// `state.toggle()`.
    pub trigger: Option<AnyElement>,
    /// Caller-supplied dropdown body element. Floated next
    /// to the trigger via `gpui::deferred` + absolute
    /// positioning by the registered `DropdownMenuRenderer`
    /// when `state.is_open()` is true.
    pub content: Option<AnyElement>,
}

pub fn dropdown_menu(
    id: impl Into<ElementId>,
    state: Entity<DropdownMenuState>,
) -> DropdownMenuProps {
    DropdownMenuProps {
        id: id.into(),
        state,
        trigger: None,
        content: None,
    }
}

impl DropdownMenuProps {
    /// Set the trigger element.
    pub fn trigger(mut self, t: AnyElement) -> Self {
        self.trigger = Some(t);
        self
    }
    /// Set the dropdown content element.
    pub fn content(mut self, c: AnyElement) -> Self {
        self.content = Some(c);
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the dropdown menu using the registered
    /// `DropdownMenuRenderer`. Returns a `Stateful<Div>` with
    /// the element id. The renderer decides trigger bg / fg /
    /// chevron based on the `state` entity.
    pub fn render(mut self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::dropdown_menu::DropdownMenuRenderer;
        use crate::renderer::markers::DropdownMenu as DropdownMenuMarker;

        let r: &Arc<dyn DropdownMenuRenderer> = cx
            .renderer_arc::<DropdownMenuMarker, dyn DropdownMenuRenderer>()
            .expect("DropdownMenuRenderer registered");
        let div = r.compose(&mut self, cx);
        self.apply(div)
    }
}
