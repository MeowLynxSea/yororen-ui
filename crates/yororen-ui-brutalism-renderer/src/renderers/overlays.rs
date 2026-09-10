//! Brutalist overlay renderers: `Modal`, `Popover`,
//! `DropdownMenu`, `Disclosure`, `Overlay`, `Menu`.

use gpui::prelude::FluentBuilder;
use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};
use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::style::{
    BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, BRUTAL_RADIUS, brutal_border_color, brutal_shadow_overlay,
};
use yororen_ui_default_renderer::animation::{AnimatedPresenceElement, fade_in_on_mount};

// =====================================================================
// Modal
// =====================================================================

pub use yororen_ui_core::renderer::modal::{ModalRenderState, ModalRenderer};

pub struct BrutalModalRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalModalRenderer {
    pub fn scrim(&self, _: &ModalRenderState, _: &Theme) -> Hsla {
        gpui::hsla(0.0, 0.0, 0.0, 0.5)
    }
    pub fn panel_bg(&self, _: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    pub fn panel_border(&self, _: &ModalRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn panel_padding(&self, _: &ModalRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.modal.padding")
            .unwrap_or(24.0) as f32;
        Edges::all(px(p))
    }
    pub fn panel_border_radius(&self, _: &ModalRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn panel_shadow_alpha(&self, _: &ModalRenderState, _: &Theme) -> f32 {
        1.0
    }
}

impl ModalRenderer for BrutalModalRenderer {
    fn compose(&self, props: &mut yororen_ui_core::headless::modal::ModalProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ModalRenderState {};
        let panel_bg = self.panel_bg(&state, theme);
        let panel_border = self.panel_border(&state, theme);
        let pad = self.panel_padding(&state, theme);
        let r = self.panel_border_radius(&state, theme);
        let shadow = brutal_shadow_overlay(theme);

        if !props.state.read(cx).is_visible() {
            return div();
        }

        let children = std::mem::take(&mut props.children);
        // Brutalism Modal renderer paints *only* the panel
        // (bg / border / padding / radius / hard offset shadow).
        // The scrim and centering are the caller's responsibility
        // — same contract as `TokenModalRenderer` in the default
        // renderer.
        let panel = gpui::div()
            .bg(panel_bg)
            .border_color(panel_border)
            .border_2()
            .p(pad.top)
            .rounded(r)
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .children(children)
            .shadow(vec![gpui::BoxShadow {
                color: shadow.color,
                blur_radius: shadow.blur,
                spread_radius: gpui::px(0.0),
                inset: false,
                offset: gpui::Point {
                    x: gpui::px(0.0),
                    y: shadow.offset_y,
                },
            }]);

        div().child(AnimatedPresenceElement::new(
            props.state.clone(),
            props.id.clone(),
            SlideDirection::Down,
            px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32),
            panel,
        ))
    }
}

// =====================================================================
// Popover
// =====================================================================

pub use yororen_ui_core::renderer::popover::{PopoverRenderState, PopoverRenderer};

pub struct BrutalPopoverRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalPopoverRenderer {
    pub fn bg(&self, _: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border(&self, _: &PopoverRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn shadow_alpha(&self, _: &PopoverRenderState, _: &Theme) -> f32 {
        1.0
    }
    pub fn border_radius(&self, _: &PopoverRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn offset(&self, _: &PopoverRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.popover.offset")
            .unwrap_or(8.0) as f32)
    }
}

impl PopoverRenderer for BrutalPopoverRenderer {
    fn compose(
        &self,
        props: &mut yororen_ui_core::headless::popover::PopoverProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = PopoverRenderState {};
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let r = self.border_radius(&state, theme);
        let alpha = self.shadow_alpha(&state, theme);
        let visible = props.state.read(cx).is_visible();
        let offset_px = self.offset(&state, theme);

        let mut outer = gpui::div().relative();

        if let Some(t) = props.trigger.take() {
            outer = outer.child(t);
        }

        if visible && let Some(c) = props.content.take() {
            // Capture outside-clicks to close the popover.
            let state_for_close = props.state.clone();
            outer = outer.on_mouse_down_out(move |_ev, _window, cx| {
                state_for_close.update(cx, |s, _cx| s.close());
            });
            let shadow = crate::style::brutal_shadow_overlay(theme);
            let panel: Div = gpui::div()
                .absolute()
                .top(offset_px)
                .left_0()
                .bg(bg)
                .border_color(border)
                .border_2()
                .rounded(r)
                .shadow(vec![gpui::BoxShadow {
                    color: gpui::hsla(0.0, 0.0, 0.0, alpha),
                    blur_radius: gpui::px(0.0),
                    spread_radius: gpui::px(0.0),
                    inset: false,
                    offset: gpui::Point {
                        x: gpui::px(0.0),
                        y: shadow.offset_y,
                    },
                }])
                .occlude()
                .child(c);
            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            // The animation wrapper is absolutely positioned at the
            // top-left of the outer relative container so the panel
            // inside keeps its original `top/left` offset.
            outer = outer.child(
                gpui::deferred(div().absolute().top_0().left_0().child(
                    AnimatedPresenceElement::new(
                        props.state.clone(),
                        (props.id.clone(), "content"),
                        SlideDirection::Down,
                        distance,
                        panel,
                    ),
                ))
                .with_priority(1),
            );
        }

        outer
    }
}

// =====================================================================
// DropdownMenu
// =====================================================================

pub use yororen_ui_core::renderer::dropdown_menu::{DropdownMenuRenderState, DropdownMenuRenderer};

pub struct BrutalDropdownMenuRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalDropdownMenuRenderer {
    pub fn trigger_bg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn trigger_hover_bg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn trigger_fg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.dropdown_menu.min_height")
            .unwrap_or(44.0) as f32)
    }
    pub fn border_radius(&self, _: &DropdownMenuRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn chevron_rotation(&self, state: &DropdownMenuRenderState, _: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

impl DropdownMenuRenderer for BrutalDropdownMenuRenderer {
    fn compose(
        &self,
        props: &mut yororen_ui_core::headless::dropdown_menu::DropdownMenuProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DropdownMenuRenderState {
            open: props.state.read(cx).is_open(),
        };
        let _border = self.trigger_bg(&state, theme);
        let _fg = self.trigger_fg(&state, theme);
        let r = self.border_radius(&state, theme);

        // Outer container is `relative` so the absolute panel
        // below is positioned relative to it.
        let mut outer = gpui::div().relative();

        // 1) Trigger — always rendered in normal flow.
        if let Some(t) = props.trigger.take() {
            outer = outer.child(t);
        }

        // 2) Body — only when visible, floated with
        //    `gpui::deferred` so it paints over subsequent
        //    sibling cells in the gallery.
        if props.state.read(cx).is_visible()
            && let Some(c) = props.content.take()
        {
            let shadow = crate::style::brutal_shadow_overlay(theme);
            let state_for_close = props.state.clone();
            // The body is a `menu` element which already paints
            // its own border + bg; the brutalism dropdown panel
            // only adds the brutalist hard offset shadow and
            // the click-outside dismissal. Avoid double borders
            // by NOT setting `border_2` / `border_color` here.
            let panel: Div = gpui::div()
                .absolute()
                .top(gpui::px(4.0))
                .left_0()
                .rounded(r)
                .shadow(vec![gpui::BoxShadow {
                    color: gpui::hsla(0.0, 0.0, 0.0, 1.0),
                    blur_radius: gpui::px(0.0),
                    spread_radius: gpui::px(0.0),
                    inset: false,
                    offset: gpui::Point {
                        x: gpui::px(0.0),
                        y: shadow.offset_y,
                    },
                }])
                .occlude()
                .on_mouse_down_out(move |_ev, _window, cx| {
                    state_for_close.update(cx, |s, _cx| s.close());
                })
                .child(c);
            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            // The animation wrapper is absolutely positioned at the
            // top-left of the outer relative container so the panel
            // inside keeps its original `top/left` offset.
            outer = outer.child(
                gpui::deferred(div().absolute().top_0().left_0().child(
                    AnimatedPresenceElement::new(
                        props.state.clone(),
                        (props.id.clone(), "body"),
                        SlideDirection::Down,
                        distance,
                        panel,
                    ),
                ))
                .with_priority(1),
            );
        }

        outer
    }
}

// =====================================================================
// Disclosure
// =====================================================================

pub use yororen_ui_core::renderer::disclosure::{DisclosureRenderState, DisclosureRenderer};

pub struct BrutalDisclosureRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalDisclosureRenderer {
    pub fn trigger_bg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn trigger_fg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn trigger_hover_bg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, _: &DisclosureRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.disclosure.min_height")
            .unwrap_or(44.0) as f32)
    }
    pub fn border_radius(&self, _: &DisclosureRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn chevron_rotation(&self, state: &DisclosureRenderState, _: &Theme) -> f32 {
        if state.open { 90.0 } else { 0.0 }
    }
    pub fn body_padding(&self, _: &DisclosureRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.disclosure.padding")
            .unwrap_or(12.0) as f32)
    }
}

impl DisclosureRenderer for BrutalDisclosureRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::disclosure::DisclosureProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DisclosureRenderState { open: props.open };
        let fg = self.trigger_fg(&state, theme);
        let chev_str = if props.open { "▼" } else { "▶" };
        // Lightweight trigger: chevron + title, no button-like
        // background / min-height / radius. Matches the default
        // renderer (a normal-weight flex row) so disclosure cells
        // look consistent with button / popover / dropdown cells.
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .text_color(fg)
            .cursor(CursorStyle::PointingHand)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .child(chev_str)
                    .child(props.title.clone()),
            )
    }
}

// =====================================================================
// Overlay
// =====================================================================

pub use yororen_ui_core::renderer::overlay::{OverlayRenderState, OverlayRenderer};

pub struct BrutalOverlayRenderer;

impl BrutalOverlayRenderer {
    pub fn scrim_color(&self, _state: &OverlayRenderState, theme: &Theme) -> Hsla {
        // Same fallback as the default renderer (50% black) so the
        // gallery shows a visible scrim even if the theme omits the
        // `surface.scrim` key.
        theme.get_color("surface.scrim").unwrap_or(gpui::hsla(0.0, 0.0, 0.0, 0.5))
    }
}

impl OverlayRenderer for BrutalOverlayRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::overlay::OverlayProps,
        cx: &App,
    ) -> gpui::Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = OverlayRenderState { open: props.open };
        let scrim = self.scrim_color(&state, theme);
        // `relative().size_full()` keeps the scrim within the
        // cell's box (the cell is now `position: relative`, see
        // `sections/mod.rs::cell`). The brutalism flavor is just
        // the scrim color — the overlay has no border / radius.
        let scrim_el = div()
            .relative()
            .size_full()
            .bg(scrim)
            .when(!props.open, |el: Div| el.invisible());

        if !props.open {
            return div().id(props.id.clone()).child(scrim_el);
        }

        let duration_ms = theme
            .get_number("motion.duration_modal_fade")
            .unwrap_or(200.0) as u64;
        let el = fade_in_on_mount(
            scrim_el,
            props.id.clone(),
            std::time::Duration::from_millis(duration_ms),
            yororen_ui_core::animation::ease_out_quad,
        );
        div().id(props.id.clone()).child(el)
    }
}

// =====================================================================
// Menu
// =====================================================================

pub use yororen_ui_core::renderer::menu::{MenuRenderState, MenuRenderer};

pub struct BrutalMenuRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalMenuRenderer {
    pub fn bg(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn border_width(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.menu.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32)
    }
    pub fn border_radius(&self, _state: &MenuRenderState, _theme: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn padding(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.menu.padding")
            .unwrap_or(4.0) as f32)
    }
    pub fn min_width(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        // Floor the menu shell so an `absolute()` panel
        // (dropdown, popover) cannot collapse below a usable
        // width. 200 px matches the brutalism convention of
        // giving chunky surfaces more breathing room than the
        // default renderer.
        px(theme
            .get_number("tokens.control.menu.min_width")
            .unwrap_or(200.0) as f32)
    }
    pub fn item_padding_x(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.menu.item_padding_x")
            .unwrap_or(10.0) as f32)
    }
    pub fn item_padding_y(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.menu.item_padding_y")
            .unwrap_or(6.0) as f32)
    }
    pub fn item_gap(&self, _state: &MenuRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.menu.gap").unwrap_or(2.0) as f32)
    }
    pub fn item_hover_bg(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        // Brutalism uses the action.neutral.hover_bg (typically
        // a vivid yellow/cyan in the brutalism palette) so the
        // hovered row is unmistakably highlighted.
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn item_hl_fg(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn group_label_fg(&self, _state: &MenuRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
    }
}

impl MenuRenderer for BrutalMenuRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::menu::MenuProps,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::headless::dropdown_menu::DropdownItem;
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = MenuRenderState {};
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let bw = self.border_width(&state, theme);
        let radius = self.border_radius(&state, theme);
        let pad = self.padding(&state, theme);
        let item_px = self.item_padding_x(&state, theme);
        let item_py = self.item_padding_y(&state, theme);
        let item_gap = self.item_gap(&state, theme);
        let item_hover_bg = self.item_hover_bg(&state, theme);
        let item_hl_fg = self.item_hl_fg(&state, theme);
        let group_label_fg = self.group_label_fg(&state, theme);
        let min_w = self.min_width(&state, theme);

        let items = props.state.read(cx).items.clone();
        let highlighted = props.state.read(cx).highlighted_index;

        // Build the menu body (flex column) with one row per
        // item. Highlighted row is painted using `item_hover_bg`
        // directly so keyboard navigation matches mouse hover
        // without an extra theme key.
        let mut body: Div = gpui::div().flex().flex_col().gap(item_gap);

        for (i, item) in items.iter().enumerate() {
            match item {
                DropdownItem::Item(menu_item) => {
                    let is_highlighted = highlighted == Some(i);
                    let state_for_pick = props.state.clone();
                    let id = menu_item.id.clone();
                    let label = menu_item.label.to_string();
                    let row_bg = if is_highlighted { item_hover_bg } else { bg };
                    let row_fg = if is_highlighted {
                        item_hl_fg
                    } else {
                        theme.get_color("content.primary").unwrap_or(item_hl_fg)
                    };
                    let mut row: Stateful<Div> = gpui::div()
                        .id(ElementId::Name(format!("brutal-menu-item-{}", i).into()))
                        .w_full()
                        .px(item_px)
                        .py(item_py)
                        .rounded(px(BRUTAL_RADIUS))
                        .bg(row_bg)
                        .text_color(row_fg)
                        .cursor(CursorStyle::PointingHand)
                        .hover(move |s| s.bg(item_hover_bg))
                        .child(label);
                    row = row.on_click(move |_ev, window, cx| {
                        let cb = state_for_pick.read(cx).on_select().cloned();
                        if let Some(f) = cb {
                            f(id.clone(), window, cx);
                        }
                    });
                    body = body.child(row);
                }
                DropdownItem::Separator => {
                    // A 2-pixel hard separator matches the
                    // brutalism divider thickness.
                    let sep = gpui::div()
                        .id(ElementId::Name(format!("brutal-menu-sep-{}", i).into()))
                        .w_full()
                        .h(px(2.0))
                        .my(px(2.0))
                        .bg(border);
                    body = body.child(sep);
                }
                DropdownItem::Group(group) => {
                    let group_label = group.label.to_string();
                    let header = gpui::div()
                        .id(ElementId::Name(format!("brutal-menu-group-{}", i).into()))
                        .w_full()
                        .px(item_px)
                        .py(px(4.0))
                        .text_color(group_label_fg)
                        .text_size(px(11.0))
                        .child(group_label);
                    body = body.child(header);
                }
            }
        }

        // Brutalism menu shell: thick black border + hard offset
        // shadow + sharp corners. The shadow uses the overlay
        // tier (largest brutalism y offset) since menus float
        // above other content.
        let shadow = brutal_shadow_overlay(theme);
        gpui::div()
            .id(props.id.clone())
            .min_w(min_w)
            .bg(bg)
            .border(bw)
            .border_color(border)
            .rounded(radius)
            .p(pad)
            .shadow(vec![gpui::BoxShadow {
                color: shadow.color,
                offset: gpui::point(px(0.0), shadow.offset_y),
                blur_radius: shadow.blur,
                spread_radius: px(0.0),
                inset: false,
            }])
            .child(body)
    }
}
