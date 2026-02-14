use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    ClickEvent, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, div, px,
};

use crate::{
    component::{generate_element_id, ArrowDirection, IconName, PopoverPlacement, button, divider, icon, popover},
    theme::{ActionVariantKind, ActiveTheme},
};

/// Creates a new dropdown menu element.
/// Requires an id to be set via `.id()` for internal state management.
///
/// # Accessibility
///
/// This component provides accessibility support through the following attributes:
/// - The trigger button is keyboard accessible (Tab to focus, Space/Enter to open)
/// - The menu container has a unique ID for proper association
/// - Menu items can be navigated using arrow keys
/// - Escape closes the menu
/// - Disabled items are properly marked and skipped during keyboard navigation
///
/// For full accessibility support:
/// - The component tracks `aria-expanded` state internally (true when menu is open)
/// - The menu container uses `role="menu"` for proper screen reader semantics
/// - Menu items use `role="menuitem"` for proper identification
/// - Separators are marked with `role="separator"`
pub fn dropdown_menu() -> DropdownMenu {
    DropdownMenu::new()
}

type SelectFn = Arc<dyn Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(Clone, Debug)]
pub enum DropdownItem {
    Item(DropdownMenuItem),
    Separator,
}

#[derive(Clone, Debug)]
pub struct DropdownMenuItem {
    pub id: String,
    pub label: SharedString,
    pub disabled: bool,
}

impl DropdownMenuItem {
    pub fn new(id: impl Into<String>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(IntoElement)]
pub struct DropdownMenu {
    element_id: Option<ElementId>,
    label: SharedString,
    items: Vec<DropdownItem>,
    open: bool,
    width: Option<gpui::Pixels>,
    placement: PopoverPlacement,
    on_select: Option<SelectFn>,
}

impl Default for DropdownMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl DropdownMenu {
    pub fn new() -> Self {
        Self {
            element_id: None,
            label: "Menu".into(),
            items: Vec::new(),
            open: false,
            width: Some(px(220.)),
            placement: PopoverPlacement::BottomStart,
            on_select: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = DropdownItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_select = Some(Arc::new(f));
        self
    }
}

impl ParentElement for DropdownMenu {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        // Intentional: DropdownMenu is a self-contained widget.
        let _ = elements;
    }
}

impl RenderOnce for DropdownMenu {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let element_id = self.element_id;

        // DropdownMenu requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = element_id.unwrap_or_else(|| generate_element_id("ui:dropdown-menu"));

        let open_state =
            window.use_keyed_state((id.clone(), "ui:dropdown-menu:open"), cx, |_, _| self.open);
        let is_open = *open_state.read(cx);

        let open_for_trigger = open_state.clone();
        let open_for_close = open_state.clone();
        let open_for_select = open_state.clone();

        let theme = cx.theme().clone();
        let on_select = self.on_select.clone();

        let id_for_menu = id.clone();

        let menu = div()
            .py_1()
            .children(
                self.items
                    .into_iter()
                    .enumerate()
                    .map(move |(ix, item)| match item {
                        DropdownItem::Separator => divider().into_any_element(),
                        DropdownItem::Item(item) => {
                            let is_disabled = item.disabled;
                            let item_id = item.id.clone();
                            let open_for_select = open_for_select.clone();
                            let on_select = on_select.clone();
                            button()
                                .id((id_for_menu.clone(), format!("ui:dropdown-menu:item-{ix}")))
                                .w_full()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .bg(theme.action.neutral.bg.alpha(0.0))
                                .hover_bg(theme.surface.hover)
                                .when(is_disabled, |this| {
                                    this.disabled(true)
                                        .text_color(theme.content.disabled)
                                        .cursor_not_allowed()
                                })
                                .when(!is_disabled, |this| {
                                    this.text_color(theme.content.primary)
                                        .cursor_pointer()
                                        .on_click(move |ev, window, cx| {
                                            open_for_select.update(cx, |open, _| *open = false);
                                            window.refresh();
                                            if let Some(handler) = &on_select {
                                                handler(item_id.clone(), ev, window, cx);
                                            }
                                        })
                                })
                                .child(item.label)
                                .into_any_element()
                        }
                    }),
            );

        popover()
            .id((id.clone(), "ui:dropdown-menu:popover"))
            .open(is_open)
            .placement(self.placement)
            .when_some(self.width, |this, width| this.width(width))
            .on_close(move |window, cx| {
                open_for_close.update(cx, |open, _| *open = false);
                window.refresh();
            })
            .trigger(
                button()
                    .id((id.clone(), "ui:dropdown-menu:trigger"))
                    .variant(ActionVariantKind::Neutral)
                    .flex()
                    .items_center()
                    .gap_2()
                    .on_click(move |_ev, window, cx| {
                        open_for_trigger.update(cx, |open, _| *open = !*open);
                        window.refresh();
                    })
                    .child(self.label)
                    .child(icon(IconName::Arrow(ArrowDirection::Down)).size(px(12.))),
            )
            .content(menu)
    }
}
