//! `TokenSplitButtonRenderer` — default `SplitButtonRenderer` impl.
//!
//! Composes a complete split-button widget (primary action +
//! chevron toggle + optional floating dropdown) by delegating
//! to the `Button`, `ListItem` and `Panel` renderers. The
//! visual logic (variant tokens, item hover, popover bg/border/
//! shadow) is read from the active theme — the headless
//! `SplitButtonProps` only carries data.
//!
//! Layout: the returned `Div` is `relative`, with the trigger
//! row as its first child and (when `state.is_open()`) an
//! `absolute` dropdown body as the second child. This puts the
//! dropdown on its own stacking layer above the surrounding
//! page flow without shifting sibling components.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    App, BoxShadow, ClickEvent, Div, ElementId, InteractiveElement, ParentElement, Pixels,
    StatefulInteractiveElement, Styled, Window, deferred, div, point, px,
};

use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::button::ButtonProps;
use yororen_ui_core::headless::dropdown_menu::DropdownItem;
use yororen_ui_core::headless::list_item::ListItemProps;
use yororen_ui_core::headless::split_button::{ClickCallback, SplitButtonProps};
use yororen_ui_core::renderer::variant::ActionVariantKind;
use yororen_ui_core::theme::Theme;

use crate::animation::AnimatedPresenceElement;

pub use yororen_ui_core::renderer::split_button::{SplitButtonRenderState, SplitButtonRenderer};

pub struct TokenSplitButtonRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenSplitButtonRenderer {
    pub fn primary_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn primary_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn chevron_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn chevron_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn chevron_hover_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    pub fn min_height(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.min_height")
            .unwrap_or(36.0) as f32)
    }
    pub fn border_radius(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
    }
    pub fn gap(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.separator_w")
            .unwrap_or(2.0) as f32)
    }
    pub fn chevron_width(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.chevron_width")
            .unwrap_or(32.0) as f32)
    }
    pub fn menu_width(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.menu_width")
            .unwrap_or(180.0) as f32)
    }
}

impl SplitButtonRenderer for TokenSplitButtonRenderer {
    fn compose(&self, props: &SplitButtonProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let (open, visible) = props
            .state
            .as_ref()
            .map(|s| {
                let s = s.read(cx);
                (s.is_open(), s.is_visible())
            })
            .unwrap_or((false, false));
        let state = SplitButtonRenderState {
            open,
            disabled: props.disabled,
            toggled: props.toggled.unwrap_or(false),
        };

        // ---- Primary button (caption only, click = props.primary) ----
        // The caption is the *selected* flyout item's label when
        // set (the pick-a-list-style ToggleSplitButton pattern),
        // else the static caption. In toggle mode both halves
        // paint the accent "checked" look via the primary action
        // variant while the toggled bit is set.
        let primary_id: ElementId = format!("{:?}-primary", props.id).into();
        let primary_variant = if state.toggled {
            ActionVariantKind::Primary
        } else {
            ActionVariantKind::Neutral
        };
        let primary = ButtonProps {
            id: primary_id,
            focus_handle: props.primary_focus.clone(),
            on_click: Some(props.primary.clone()),
            disabled: props.disabled,
            clickable: true,
            variant: primary_variant,
            caption: props.display_caption(),
            icon: None,
            icon_size: px(16.),
        }
        .render(cx);

        // ---- Chevron button (toggles dropdown_state) ----
        let state_for_chevron = props.state.clone();
        let chevron_click: ClickCallback =
            Arc::new(move |_ev: &ClickEvent, _w: &mut Window, cx: &mut App| {
                if let Some(s) = state_for_chevron.as_ref() {
                    // Notify so the open/close flip repaints even
                    // when the caller's handlers don't notify.
                    s.update(cx, |st, cx| {
                        st.toggle();
                        cx.notify();
                    });
                }
            });
        let chevron_label = if open { "▴" } else { "▾" };
        let chevron_id: ElementId = format!("{:?}-chevron", props.id).into();
        let chevron_w = self.chevron_width(&state, theme);
        let chevron = ButtonProps {
            id: chevron_id,
            focus_handle: props.chevron_focus.clone(),
            on_click: Some(chevron_click),
            disabled: props.disabled,
            clickable: true,
            variant: primary_variant,
            caption: Some(chevron_label.into()),
            icon: None,
            icon_size: px(16.),
        }
        .render(cx)
        .w(chevron_w)
        .px(px(0.));

        let gap = self.gap(&state, theme);
        let trigger_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
            .child(primary)
            .child(chevron);

        // Outside-press dismissal hangs off the menu panel (see
        // below), NOT on this wrapper: gpui's `on_mouse_down_out`
        // is a geometric check of the element's own hitbox, and the
        // wrapper's hitbox only covers the trigger row — the
        // absolute, deferred menu hangs below it. A wrapper-level
        // `_out` therefore fired on every menu-item press, closing
        // the menu mid-click; the exit-animating items then slid
        // out from under the pointer and gpui dropped the click, so
        // `on_select` never ran (the "picked an item but nothing
        // switched" bug). Honoured only when the caller opted in
        // via `dismiss_on_outside_click`.
        let dismiss_outside = props
            .state
            .as_ref()
            .map(|s| s.read(cx).dismiss_on_outside_click)
            .unwrap_or(true);
        // A press on the trigger row (primary / chevron) must not
        // count as "outside" either: the chevron's click handler
        // toggles the open state on mouse-up, so dismissing on
        // mouse-down would close-then-reopen the menu. Capture
        // listeners run in paint order (ancestors first), so this
        // marker on the wrapper executes ahead of the menu's `_out`
        // and suppresses it for trigger presses.
        let trigger_press = Rc::new(Cell::new(false));

        // ---- Dropdown body (only when open) ----
        // Wrapped in `gpui::deferred(...)` so the popover paints
        // *after* every other sibling in the tree. Without this,
        // `.absolute()` only removes the element from layout flow
        // — paint order remains DOM order, so any later sibling
        // (e.g. the next row in the section) would draw on top of
        // the menu and you'd see "through" it.
        let mut root = div().relative().child(trigger_row);
        if dismiss_outside {
            let mark_trigger_press = trigger_press.clone();
            root = root.capture_any_mouse_down(move |_ev, _w, _cx| {
                mark_trigger_press.set(true);
            });
        }
        if visible {
            // Dropdown bg prefers `surface.popover` (a dedicated
            // contrast colour the JSON theme can override) and
            // falls back to `surface.raised` so older theme
            // packages still render with a sensible elevation.
            let panel_bg = theme
                .get_color("surface.popover")
                .or_else(|| theme.get_color("surface.raised"))
                .unwrap_or_default();
            let panel_border = theme.get_color("border.default").unwrap_or_default();
            let panel_radius = px(theme.get_number("tokens.radii.lg").unwrap_or(8.0) as f32);
            let panel_pad = px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32);
            let item_hover_bg = theme.get_color("surface.hover").unwrap_or_default();
            let shadow_color = theme.get_color("shadow.elevation_2").unwrap_or_default();
            let divider_color = theme.get_color("border.divider").unwrap_or_default();
            let menu_w = self.menu_width(&state, theme);
            let min_h = self.min_height(&state, theme);
            // Place the menu just below the trigger row.
            let menu_offset = min_h + px(4.);

            let mut menu = div()
                .absolute()
                .top(menu_offset)
                .left_0()
                .w(menu_w)
                .bg(panel_bg)
                .border_1()
                .border_color(panel_border)
                .rounded(panel_radius)
                .p(panel_pad)
                .flex()
                .flex_col()
                .gap(px(2.))
                .shadow(vec![BoxShadow {
                    color: shadow_color,
                    offset: point(px(0.), px(4.)),
                    blur_radius: px(12.),
                    spread_radius: px(0.),
                }])
                // popover pattern: occlude (the
                // `InteractiveElement` trait method) blocks
                // events from reaching elements painted behind
                // the menu (stops a click on an option from
                // also firing on the cell directly below the
                // split button).
                .occlude();

            // Dismiss on outside presses. On the panel itself,
            // item presses land inside its hitbox and never fire
            // `_out`; the wrapper's capture marker above exempts
            // trigger presses so the chevron keeps toggling.
            if dismiss_outside {
                let state_for_close = props.state.clone();
                let suppress = trigger_press.clone();
                menu = menu.on_mouse_down_out(move |_ev, _w, cx| {
                    if suppress.replace(false) {
                        return;
                    }
                    if let Some(st) = state_for_close.as_ref() {
                        st.update(cx, |s, cx| {
                            s.close();
                            cx.notify();
                        });
                    }
                });
            }

            for it in &props.items {
                match it {
                    DropdownItem::Item(item) => {
                        let item_id_str = item.id.clone();
                        let item_label = item.label.clone();
                        let item_disabled = item.disabled;
                        let state_for_click = props.state.clone();
                        let on_select_for_click = props.on_select.clone();
                        let item_id_for_callback = item_id_str.clone();

                        let row_id: ElementId =
                            format!("{:?}-item-{}", props.id, item_id_str).into();
                        let is_selected_item = props.selected_item.as_ref() == Some(&item_id_str);
                        let list_item_el = ListItemProps {
                            id: row_id,
                            title: item_label,
                            description: None,
                            leading_icon: None,
                            trailing_icon: None,
                            selected: is_selected_item,
                            disabled: item_disabled,
                            on_click: None,
                        }
                        .render(cx);

                        // The selected flyout item keeps the
                        // `action.primary` look (matching the select
                        // dropdown's selected option); plain items
                        // blend with the menu container. Hover on the
                        // selected item stays within the primary
                        // pair instead of washing it out.
                        let (item_base, item_hover_target) = if is_selected_item {
                            (
                                theme.get_color("action.primary.bg").unwrap_or(panel_bg),
                                theme
                                    .get_color("action.primary.hover_bg")
                                    .unwrap_or(item_hover_bg),
                            )
                        } else {
                            (panel_bg, item_hover_bg)
                        };
                        let item_el = if !item_disabled {
                            list_item_el
                                .w_full()
                                // Override list_item's `surface.base`
                                // default so items blend with the
                                // menu container instead of stamping
                                // a contrasting rectangle on it.
                                .bg(item_base)
                                .cursor_pointer()
                                .hover(move |s| s.bg(item_hover_target))
                                .on_click(move |_ev, window, cx| {
                                    if let Some(st) = state_for_click.as_ref() {
                                        st.update(cx, |s, cx| {
                                            s.close();
                                            cx.notify();
                                        });
                                    }
                                    if let Some(cb) = on_select_for_click.as_ref() {
                                        cb(item_id_for_callback.clone(), window, cx);
                                    }
                                })
                        } else {
                            list_item_el.w_full().bg(panel_bg)
                        };
                        menu = menu.child(item_el);
                    }
                    DropdownItem::Separator => {
                        menu = menu.child(div().h(px(1.)).bg(divider_color).my(px(2.)));
                    }
                    DropdownItem::Group(_) => {
                        // Groups are not rendered specially in v0.3.
                    }
                }
            }

            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            let state_entity = props
                .state
                .clone()
                .expect("visible implies state is present");
            // The animation wrapper is absolutely positioned at the
            // top-left of the root relative container so the menu
            // inside keeps its original `top/left` offset.
            root.child(
                deferred(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .child(AnimatedPresenceElement::new(
                            state_entity,
                            (props.id.clone(), "menu"),
                            SlideDirection::Down,
                            distance,
                            div().child(menu),
                        )),
                )
                .with_priority(1),
            )
        } else {
            root
        }
    }
}

pub fn arc_split_button<T: SplitButtonRenderer + 'static>(r: T) -> Arc<dyn SplitButtonRenderer> {
    Arc::new(r)
}
