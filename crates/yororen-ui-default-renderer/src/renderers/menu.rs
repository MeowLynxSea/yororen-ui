//! `TokenMenuRenderer` — default `MenuRenderer` impl.
//!
//! Paints a rounded panel with a border and subtle shadow — the
//! typical dropdown / context-menu shell. Iterates
//! `props.state.items` to render each item, separator, or
//! group; clicking an item fires `state.on_select` and
//! closes the menu via `state.select_highlighted` semantics
//! (or just `on_select` for menus without highlight tracking).

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};
use yororen_ui_core::headless::dropdown_menu::DropdownItem;
use yororen_ui_core::headless::menu::MenuProps;
use yororen_ui_core::headless::scroll_view::ScrollViewProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::menu::{MenuRenderState, MenuRenderer};

pub struct TokenMenuRenderer;

impl TokenMenuRenderer {
    pub fn bg(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn border_radius(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
    }
    pub fn padding(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(4.0) as f32)
    }
    pub fn min_width(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        // Floor the menu shell so an `absolute()` panel
        // (dropdown, popover) cannot collapse below a usable
        // width. Without this, a flex-col body whose items
        // don't `.w_full()` shrinks to the widest label, which
        // can become 1-character-wide if any item happens to be
        // shorter in the layout context the dropdown panel
        // inherits. 180 px is the conventional menu width and
        // matches the brutalism renderer.
        px(theme
            .get_number("tokens.control.menu.min_width")
            .unwrap_or(180.0) as f32)
    }
    pub fn shadow_alpha(&self, _state: &MenuRenderState, _theme: &Theme) -> f32 {
        0.12
    }
    pub fn item_hover_bg(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
}

impl MenuRenderer for TokenMenuRenderer {
    fn compose(&self, props: &MenuProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = MenuRenderState {};
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let radius = self.border_radius(&state, theme);
        let pad = self.padding(&state, theme);
        let alpha = self.shadow_alpha(&state, theme);
        let item_hover = self.item_hover_bg(&state, theme);
        let min_w = self.min_width(&state, theme);

        let items = props.state.read(cx).items.clone();
        let highlighted = props.state.read(cx).highlighted_index;
        let scroll = props.state.read(cx).scroll.clone();
        let max_h = px(theme
            .get_number("tokens.control.menu.max_visible_height")
            .unwrap_or(320.0) as f32);

        // Rows are collected as `AnyElement`s so they can be moved
        // into the embedded scroll view as its *direct* children —
        // `ScrollHandle::scroll_to_item` addresses direct children,
        // which is what the keyboard reveal relies on.
        let mut rows: Vec<gpui::AnyElement> = Vec::with_capacity(items.len());

        for (i, item) in items.iter().enumerate() {
            match item {
                DropdownItem::Item(menu_item) => {
                    let is_highlighted = highlighted == Some(i);
                    let state_for_pick = props.state.clone();
                    let id = menu_item.id.clone();
                    let label = menu_item.label.to_string();
                    let row_bg = if is_highlighted {
                        item_hover
                    } else {
                        theme.get_color("surface.base").unwrap_or_default()
                    };
                    let mut row: Stateful<Div> = div()
                        .id(ElementId::Name(format!("menu-item-{}", i).into()))
                        .w_full()
                        .px(px(8.0))
                        .py(px(6.0))
                        .rounded(px(4.0))
                        .bg(row_bg)
                        .cursor(CursorStyle::PointingHand)
                        .hover(move |s| s.bg(item_hover))
                        .child(label);
                    row = row.on_click(move |_ev, window, cx| {
                        // Fire `on_select` directly. The menu
                        // also exposes `select_highlighted` for
                        // keyboard navigation; the click path
                        // bypasses the highlight index.
                        let cb = state_for_pick.read(cx).on_select().cloned();
                        if let Some(f) = cb {
                            f(id.clone(), window, cx);
                        }
                    });
                    rows.push(::gpui::IntoElement::into_any_element(row));
                }
                DropdownItem::Separator => {
                    let sep = div()
                        .id(ElementId::Name(format!("menu-sep-{}", i).into()))
                        .w_full()
                        .h(px(1.0))
                        .my(px(2.0))
                        .bg(border);
                    rows.push(::gpui::IntoElement::into_any_element(sep));
                }
                DropdownItem::Group(group) => {
                    let group_label = group.label.to_string();
                    let header = div()
                        .id(ElementId::Name(format!("menu-group-{}", i).into()))
                        .w_full()
                        .px(px(8.0))
                        .py(px(4.0))
                        .text_color(theme.get_color("content.tertiary").unwrap_or_default())
                        .text_size(px(11.0))
                        .child(group_label);
                    rows.push(::gpui::IntoElement::into_any_element(header));
                }
            }
        }

        div()
            .id(props.id.clone())
            .min_w(min_w)
            .bg(bg)
            .border_1()
            .border_color(border)
            .rounded(radius)
            .p(pad)
            .shadow(vec![gpui::BoxShadow {
                color: gpui::hsla(0.0, 0.0, 0.0, alpha),
                blur_radius: px(12.0),
                spread_radius: px(0.0),
                inset: false,
                offset: gpui::Point {
                    x: px(0.0),
                    y: px(4.0),
                },
            }])
            .child(ScrollViewProps::compose_embedded(
                ElementId::Name(format!("{:?}-scroll", props.id).into()),
                &scroll,
                Some(max_h),
                Some(px(2.)),
                rows,
                cx,
            ))
    }
}

pub fn arc_menu<T: MenuRenderer + 'static>(r: T) -> Arc<dyn MenuRenderer> {
    Arc::new(r)
}
