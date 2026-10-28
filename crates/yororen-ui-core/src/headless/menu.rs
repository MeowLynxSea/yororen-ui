//! Headless `menu` — like `dropdown_menu` but rendered without a
//! trigger (used inside `popover`s or as a context menu body).

use crate::renderer::RendererContext;
use std::sync::Arc;

use gpui::{App, AppContext, Div, ElementId, Entity, InteractiveElement, SharedString, Stateful};

use super::dropdown_menu::{DropdownItem, DropdownMenuItem, DropdownSelectCallback};
use super::list_navigable::ListNavigable;

#[derive(Clone)]
pub struct MenuState {
    pub highlighted_index: Option<usize>,
    /// Scroll controller for the menu list (see
    /// `ListboxState::scroll`).
    pub scroll: crate::headless::scroll_view::ScrollViewController,
    pub items: Vec<DropdownItem>,
    on_select: Option<DropdownSelectCallback>,
}

impl MenuState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            highlighted_index: None,
            scroll: crate::headless::scroll_view::ScrollViewController::new(),
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

    pub fn set_items(&mut self, items: Vec<DropdownItem>) {
        self.items = items;
    }
    pub fn highlight_next(&mut self) {
        // The shared `highlight_next` algorithm wraps at the
        // ends and uses `is_selectable` to filter out slots
        // (separators here). To preserve the original "skip
        // past runs of separators" behaviour — a separator-only
        // section at the end of the menu must not leave the
        // highlight stuck on the last separator — we hand-roll
        // the loop. `is_selectable` is the same predicate the
        // trait uses, so this stays in lock-step with the
        // shared algorithm when no separators are involved.
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let mut candidate = match self.highlighted_index {
            Some(i) => i + 1,
            None => 0,
        };
        while candidate < len && !self.is_selectable(candidate) {
            candidate += 1;
        }
        if candidate < len {
            self.highlighted_index = Some(candidate);
        }
    }
    pub fn highlight_prev(&mut self) {
        // Symmetric to `highlight_next`: the shared
        // `highlight_prev` uses `is_selectable` and would refuse
        // to land on slot 0 even if it is a separator. The
        // original hand-rolled loop stops at 0 unconditionally,
        // which is the behaviour we keep here so callers that
        // rely on "↓ wraps around to the top non-separator,
        // ↑ wraps around to slot 0 even if it is a separator"
        // see no change.
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let mut candidate = match self.highlighted_index {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        while candidate > 0 && !self.is_selectable(candidate) {
            candidate -= 1;
        }
        self.highlighted_index = Some(candidate);
    }
    pub fn set_on_select<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_select = Some(Arc::new(f));
    }
    /// Read-only access to the stored `on_select` callback.
    /// Renderers use this to fire selection from each menu
    /// row's click handler.
    pub fn on_select(&self) -> Option<&DropdownSelectCallback> {
        self.on_select.as_ref()
    }
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(DropdownItem::Item(DropdownMenuItem { id, .. })) = self.items.get(i)
        {
            let id = id.clone();
            if let Some(f) = &self.on_select {
                f(id, window, cx);
            }
        }
    }
}

impl ListNavigable for MenuState {
    fn options_len(&self) -> usize {
        self.items.len()
    }
    fn highlighted_index(&self) -> Option<usize> {
        self.highlighted_index
    }
    fn set_highlighted(&mut self, i: usize) {
        self.highlighted_index = Some(i);
    }
    /// Separators and (non-navigable) group headers are not
    /// selectable slots — `highlight_prev` will skip them when
    /// wrapping around. Group headers and `Item`s both occupy a
    /// slot but only `Item` rows accept a click, so the
    /// trait-level selectability filter only excludes
    /// separators (group headers are technically highlightable
    /// in the existing menu semantics, but never clickable).
    fn is_selectable(&self, i: usize) -> bool {
        i < self.items.len() && !matches!(self.items[i], DropdownItem::Separator)
    }
}

#[derive(Clone)]
pub struct MenuProps {
    pub id: ElementId,
    pub state: Entity<MenuState>,
}

pub fn menu(id: impl Into<ElementId>, state: Entity<MenuState>) -> MenuProps {
    MenuProps {
        id: id.into(),
        state,
    }
}

impl MenuProps {
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the menu shell through the registered `MenuRenderer`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let r = cx
            .renderer_arc::<crate::renderer::markers::Menu, dyn crate::renderer::menu::MenuRenderer>()
            .expect("MenuRenderer registered");
        // Caller appends menu items via `.child(...)` after this call.
        r.compose(&self, cx)
    }
}
