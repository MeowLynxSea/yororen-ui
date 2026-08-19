//! `WinUIMenuRenderer` — default `MenuRenderer` impl.
//!
//! Paints a rounded panel with a border and subtle shadow — the
//! typical dropdown / context-menu shell. Iterates
//! `props.state.items` to render each item, separator, or
//! group; clicking an item fires `state.on_select` and
//! closes the menu via `state.select_highlighted` semantics
//! (or just `on_select` for menus without highlight tracking).

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};
use yororen_ui_core::animation::AnimationConfig;
use yororen_ui_core::headless::dropdown_menu::DropdownItem;
use yororen_ui_core::headless::menu::MenuProps;
use yororen_ui_core::theme::Theme;

use crate::animation::{AnimatedStateElement, lerp_hsla, set_interaction_hovered};

pub use yororen_ui_core::renderer::menu::{MenuRenderState, MenuRenderer};

pub struct WinUIMenuRenderer;

impl WinUIMenuRenderer {
    pub fn bg(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.popover")
            .or_else(|| theme.get_color("surface.raised"))
            .unwrap_or_default()
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

impl MenuRenderer for WinUIMenuRenderer {
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

        // Build the menu body (flex column) with one row per item.
        let mut body: Div = div().flex().flex_col().gap(px(2.0));

        for (i, item) in items.iter().enumerate() {
            match item {
                DropdownItem::Item(menu_item) => {
                    let is_highlighted = highlighted == Some(i);
                    let state_for_pick = props.state.clone();
                    let id = menu_item.id.clone();
                    let label = menu_item.label.to_string();
                    let row_id = ElementId::Name(format!("menu-item-{}", i).into());
                    let hover_row_id = row_id.clone();
                    let row_hover = item_hover;
                    let row_base = theme.get_color("surface.base").unwrap_or_default();
                    let config =
                        AnimationConfig::default().with_duration(Duration::from_millis(100));

                    let mut row: Stateful<Div> = div()
                        .id(row_id.clone())
                        .relative()
                        .w_full()
                        .px(px(8.0))
                        .py(px(6.0))
                        .rounded(px(4.0))
                        .cursor(CursorStyle::PointingHand)
                        .on_hover(move |hovered, _win, cx| {
                            set_interaction_hovered(cx, hover_row_id.clone(), *hovered)
                        });

                    let fill = AnimatedStateElement::new(
                        (row_id.clone(), "fill"),
                        row_id.clone(),
                        is_highlighted,
                        div().absolute().inset_0().rounded(px(4.0)),
                        config,
                        move |d: Div, hover, _pressed, checked| {
                            let base = lerp_hsla(row_base, row_hover, checked);
                            let next = lerp_hsla(base, row_hover, hover);
                            d.bg(next)
                        },
                    );
                    row = row
                        .child(fill)
                        .child(label)
                        .on_click(move |_ev, window, cx| {
                            // Fire `on_select` directly. The menu
                            // also exposes `select_highlighted` for
                            // keyboard navigation; the click path
                            // bypasses the highlight index.
                            let cb = state_for_pick.read(cx).on_select().cloned();
                            if let Some(f) = cb {
                                f(id.clone(), window, cx);
                            }
                        });
                    body = body.child(row);
                }
                DropdownItem::Separator => {
                    let sep = div()
                        .id(ElementId::Name(format!("menu-sep-{}", i).into()))
                        .w_full()
                        .h(px(1.0))
                        .my(px(2.0))
                        .bg(border);
                    body = body.child(sep);
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
                    body = body.child(header);
                }
            }
        }

        div()
            .id(props.id.clone())
            .min_w(min_w)
            .bg(bg)
            .text_color(theme.get_color("content.primary").unwrap_or_default())
            .border_1()
            .border_color(border)
            .rounded(radius)
            .p(pad)
            .shadow(vec![gpui::BoxShadow {
                color: gpui::hsla(0.0, 0.0, 0.0, alpha),
                blur_radius: px(12.0),
                spread_radius: px(0.0),
                offset: gpui::Point {
                    x: px(0.0),
                    y: px(4.0),
                },
            }])
            .child(body)
    }
}

pub fn arc_menu<T: MenuRenderer + 'static>(r: T) -> Arc<dyn MenuRenderer> {
    Arc::new(r)
}
