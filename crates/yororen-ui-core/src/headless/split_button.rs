//! Headless `split_button` — primary action + chevron-triggered
//! dropdown menu.
//!
//! Data contract (pure data, no visual decisions):
//!
//! - `primary`            — click handler for the main label.
//! - `caption`            — text for the main label.
//! - `items`              — dropdown menu items (reused from
//!   `dropdown_menu::DropdownItem`).
//! - `on_select`          — fires when the user picks an item.
//! - `state`              — the `Entity<DropdownMenuState>` that
//!   stores `open` + highlighted index. Caller mints it (typically
//!   in `App::new`) so the toggle survives across re-paints.
//! - `disabled`           — disables both primary + chevron.
//! - `toggled`            — `Some(_)` turns the split button into
//!   a *toggle* split button (WinUI `ToggleSplitButton`): the
//!   whole trigger paints the accent "checked" look while the
//!   value is `true`. The caller owns the bit — `primary` fires
//!   as usual and the caller flips its state in response.
//!   `None` (default) keeps the classic action split button.
//! - `selected_item`      — id of the flyout item currently
//!   chosen; the primary half's caption is replaced with that
//!   item's label (the "pick a list style" WinUI pattern).
//!   Picking an item usually sets this AND `toggled = true`.
//!
//! The factory mints two focus handles (`primary_focus` /
//! `chevron_focus`) so the renderer can compose two underlying
//! `button` props without needing `&mut App` at render time.
//!
//! Example (end-user app code):
//!
//! ```ignore
//! split_button("save-1", |_, _, cx| { /* primary save */ }, cx)
//!     .state(app.split_dd_state.clone())
//!     .caption("Save")
//!     .items(vec![
//!         DropdownItem::Item(DropdownMenuItem::new("save", "Save")),
//!         DropdownItem::Item(DropdownMenuItem::new("save_as", "Save as…")),
//!     ])
//!     .on_select(|id, _w, cx| { /* dispatch */ })
//!     .render(cx)
//! ```

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, Entity, FocusHandle, InteractiveElement, SharedString,
    Stateful, Window,
};

use crate::headless::dropdown_menu::{DropdownItem, DropdownMenuState};

pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;
pub type SelectCallback = Arc<dyn Fn(SharedString, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct SplitButtonProps {
    pub id: ElementId,
    pub primary: ClickCallback,
    pub disabled: bool,
    pub caption: Option<SharedString>,
    pub items: Vec<DropdownItem>,
    pub on_select: Option<SelectCallback>,
    pub state: Option<Entity<DropdownMenuState>>,
    /// `Some(_)` opts the split button into *toggle* mode
    /// (WinUI `ToggleSplitButton`): the whole trigger paints the
    /// accent "checked" look while the contained value is `true`.
    /// `None` (default) = classic action split button.
    pub toggled: Option<bool>,
    /// Id of the flyout item currently chosen (toggle split
    /// buttons replace the primary half's caption with the
    /// selected option's label — e.g. the WinUI bullet-list
    /// ToggleSplitButton swaps its glyph when the user picks a
    /// different list style from the flyout). `None` keeps the
    /// static `caption`. Picking an item typically sets this AND
    /// `toggled` to `Some(true)`; clicking the primary half only
    /// flips the checked bit (the selection is retained).
    pub selected_item: Option<SharedString>,
    /// Focus handle for the primary button (minted in factory).
    /// The renderer reuses this when composing the inner
    /// `ButtonProps` so the same id maps to a stable focus.
    pub primary_focus: FocusHandle,
    /// Focus handle for the chevron button (minted in factory).
    pub chevron_focus: FocusHandle,
}

pub fn split_button(
    id: impl Into<ElementId>,
    primary: impl 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    cx: &mut App,
) -> SplitButtonProps {
    SplitButtonProps {
        id: id.into(),
        primary: Arc::new(primary),
        disabled: false,
        caption: None,
        items: Vec::new(),
        on_select: None,
        state: None,
        toggled: None,
        selected_item: None,
        primary_focus: cx.focus_handle(),
        chevron_focus: cx.focus_handle(),
    }
}

impl SplitButtonProps {
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
    pub fn items(mut self, items: Vec<DropdownItem>) -> Self {
        self.items = items;
        self
    }
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut Window, &mut App),
    {
        self.on_select = Some(Arc::new(f));
        self
    }
    pub fn state(mut self, s: Entity<DropdownMenuState>) -> Self {
        self.state = Some(s);
        self
    }
    /// Turn the split button into a *toggle* split button with
    /// the given on/off value. The trigger renders the accent
    /// "checked" look while `v` is `true`; `primary` still fires
    /// on click and the caller flips its own state in response
    /// (same one-way data flow as `toggle_button`).
    pub fn toggled(mut self, v: bool) -> Self {
        self.toggled = Some(v);
        self
    }
    /// Replace the primary half's caption with the label of the
    /// chosen flyout item. Pass the item's id (see
    /// [`SplitButtonProps::selected_item`]); when the id is not
    /// found in `items` the static `caption` is kept.
    pub fn selected_item(mut self, v: impl Into<Option<SharedString>>) -> Self {
        self.selected_item = v.into();
        self
    }
    /// The caption the renderer should paint on the primary
    /// half: the selected item's label when set (and found in
    /// `items`), else the static `caption`.
    pub fn display_caption(&self) -> Option<SharedString> {
        let from_item = self.selected_item.as_ref().and_then(|sel| {
            self.items.iter().find_map(|it| match it {
                DropdownItem::Item(i) if &i.id == sel => Some(i.label.clone()),
                _ => None,
            })
        });
        from_item.or_else(|| self.caption.clone())
    }
    /// `true` when the caller opted into toggle mode via
    /// [`SplitButtonProps::toggled`].
    pub fn is_toggle(&self) -> bool {
        self.toggled.is_some()
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the split button using the registered
    /// `SplitButtonRenderer`. Returns a `Stateful<Div>` whose
    /// children are the trigger row (primary + chevron) and,
    /// when `state.open`, an absolutely-positioned dropdown
    /// body composed of `panel` + `list_item` renderers.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::SplitButton as SplitButtonMarker;
        use crate::renderer::split_button::SplitButtonRenderer;

        let r: &Arc<dyn SplitButtonRenderer> = cx
            .renderer_arc::<SplitButtonMarker, dyn SplitButtonRenderer>()
            .expect("SplitButtonRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
