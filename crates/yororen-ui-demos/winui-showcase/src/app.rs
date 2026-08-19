//! `WinuiApp` — the demo's root view + WinUI Gallery-style frame.
//!
//! Dark, Mica-flavoured WinUI app frame:
//!
//! - title bar with working back / sidebar-toggle buttons and a
//!   search box that filters the navigation,
//! - a left navigation sidebar with WinUI groups (line chevrons),
//!   a **single animated selected rail**, and fade-in group items,
//! - a scrollable content area that stays a fixed height so the
//!   sidebar is consistent across pages.

use std::time::Duration;

use gpui::{
    AnimationExt, AnyElement, App, Context, Div, Element, ElementId, Entity, GlobalElementId,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, ParentElement, Pixels, Render,
    Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};

use yororen_ui::headless::combo_box::ComboBoxState;
use yororen_ui::headless::dropdown_menu::DropdownMenuState;
use yororen_ui::headless::icon::{IconSource, icon};
use yororen_ui::headless::label::label;
use yororen_ui::headless::listbox::{ListboxOption, ListboxState};
use yororen_ui::headless::menu::MenuState;
use yororen_ui::headless::search_input::search_input;
use yororen_ui::headless::select::{SelectOption, SelectState};
use yororen_ui::theme::ActiveTheme;

use crate::pages;

/// The five pages reachable from the sidebar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WinuiPage {
    #[default]
    Home,
    Buttons,
    Inputs,
    Toggles,
    Lists,
}

impl WinuiPage {
    pub fn label(self) -> &'static str {
        match self {
            WinuiPage::Home => "Home",
            WinuiPage::Buttons => "Buttons & Actions",
            WinuiPage::Inputs => "Inputs",
            WinuiPage::Toggles => "Toggles & Sliders",
            WinuiPage::Lists => "Lists & Menus",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            WinuiPage::Home => "WinUI on Rust Gallery",
            WinuiPage::Buttons => "Buttons & Actions",
            WinuiPage::Inputs => "Inputs",
            WinuiPage::Toggles => "Toggles & Sliders",
            WinuiPage::Lists => "Lists & Menus",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            WinuiPage::Home => "Bring WinUI experience to the web.",
            WinuiPage::Buttons => "Standard, primary, danger, icon, toggle and split buttons.",
            WinuiPage::Inputs => "Text, password, number, search, path and multiline inputs.",
            WinuiPage::Toggles => "Checkbox, switch, radio and slider — all live.",
            WinuiPage::Lists => "Select, combo box, listbox and dropdown menus.",
        }
    }

    pub fn icon_name(self) -> &'static str {
        match self {
            WinuiPage::Home => "user",
            WinuiPage::Buttons => "check",
            WinuiPage::Inputs => "pencil",
            WinuiPage::Toggles => "arrow-left",
            WinuiPage::Lists => "folder",
        }
    }
}

/// All interactive state for the demo app.
#[allow(dead_code)]
pub struct WinuiApp {
    // ---- navigation ----
    pub page: WinuiPage,
    pub page_history: Vec<WinuiPage>,
    pub sidebar_collapsed: bool,
    /// Live text of the title-bar search box (filters the sidebar).
    pub nav_search: String,
    /// Which sidebar groups are expanded (`"basic"`).
    pub groups_open: std::collections::HashSet<&'static str>,

    // ---- composite `Entity<XxxState>` ----
    pub select_state: Entity<SelectState>,
    pub combo_state: Entity<ComboBoxState>,
    pub menu_state: Entity<MenuState>,
    pub dropdown_state: Entity<DropdownMenuState>,
    pub split_dd_state: Entity<DropdownMenuState>,
    pub listbox_state: Entity<ListboxState>,

    // ---- plain values ----
    pub text: String,
    pub password: String,
    pub number: f64,
    pub search: String,
    pub file_path: String,
    pub text_area: String,

    pub checkbox: bool,
    pub switch_on: bool,
    pub radio: usize,
    pub slider: f32,
    pub toggle_sel: bool,
    pub primary_clicks: usize,
    pub split_action: usize,

    pub select_value: String,
    pub combo_value: String,
    pub listbox_value: String,
    pub dropdown_value: String,
}

impl WinuiApp {
    #[allow(clippy::explicit_auto_deref)]
    pub fn new(cx: &mut Context<Self>) -> Self {
        let select_state = SelectState::new(&mut **cx);
        let combo_state = ComboBoxState::new(&mut **cx);
        let menu_state = MenuState::new(&mut **cx);
        let dropdown_state = DropdownMenuState::new(&mut **cx);
        let split_dd_state = DropdownMenuState::new(&mut **cx);
        let listbox_state = ListboxState::new(&mut **cx);

        select_state.update(cx, |s, _cx| {
            s.set_options(vec![
                SelectOption::new("apple", "Apple"),
                SelectOption::new("banana", "Banana"),
                SelectOption::new("cherry", "Cherry"),
                SelectOption::new("durian", "Durian"),
            ]);
        });
        combo_state.update(cx, |s, _cx| {
            s.set_options(vec![
                yororen_ui::headless::combo_box::ComboBoxOption::new("rust", "Rust"),
                yororen_ui::headless::combo_box::ComboBoxOption::new("go", "Go"),
                yororen_ui::headless::combo_box::ComboBoxOption::new("python", "Python"),
                yororen_ui::headless::combo_box::ComboBoxOption::new("zig", "Zig"),
            ]);
        });
        menu_state.update(cx, |s, _cx| {
            use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
            s.set_items(vec![
                DropdownItem::Item(DropdownMenuItem::new("profile", "Profile")),
                DropdownItem::Item(DropdownMenuItem::new("settings", "Settings")),
                DropdownItem::Separator,
                DropdownItem::Item(DropdownMenuItem::new("logout", "Log out")),
            ]);
        });
        listbox_state.update(cx, |s, _cx| {
            s.set_options(vec![
                ListboxOption::new("apple", "Apple"),
                ListboxOption::new("banana", "Banana"),
                ListboxOption::new("cherry", "Cherry"),
                ListboxOption::new("durian", "Durian"),
            ]);
        });

        let mut groups_open = std::collections::HashSet::new();
        groups_open.insert("basic");

        Self {
            page: WinuiPage::Home,
            page_history: Vec::new(),
            sidebar_collapsed: false,
            nav_search: String::new(),
            groups_open,
            select_state,
            combo_state,
            menu_state,
            dropdown_state,
            split_dd_state,
            listbox_state,
            text: String::new(),
            password: String::new(),
            number: 40.0,
            search: String::new(),
            file_path: String::new(),
            text_area: String::new(),
            checkbox: true,
            switch_on: true,
            radio: 0,
            slider: 60.0,
            toggle_sel: false,
            primary_clicks: 0,
            split_action: 0,
            select_value: String::new(),
            combo_value: String::new(),
            listbox_value: String::new(),
            dropdown_value: String::new(),
        }
    }

    // -----------------------------------------------------------------
    // Title bar
    // -----------------------------------------------------------------

    fn render_titlebar(&self, window: &mut Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let entity_change = cx.entity().clone();
        let entity_clear = cx.entity().clone();
        let entity_back = cx.entity().clone();
        let entity_toggle = cx.entity().clone();
        let (accent, accent_text, secondary, hover_bg) = {
            let t = cx.theme();
            (
                t.get_color("winui.accent")
                    .unwrap_or_else(|| t.get_color("action.primary.bg").unwrap_or_default()),
                t.get_color("winui.accent_text")
                    .unwrap_or(gpui::hsla(0., 0., 0., 1.)),
                t.get_color("content.secondary").unwrap_or_default(),
                t.get_color("winui.ctrl_fill_hover")
                    .unwrap_or_else(|| t.get_color("surface.hover").unwrap_or_default()),
            )
        };

        let brand_tile = div()
            .size(px(24.0))
            .rounded(px(5.0))
            .bg(accent)
            .flex()
            .items_center()
            .justify_center()
            .text_color(accent_text)
            .child("W");

        let search = search_input("winui-tb-search")
            .placeholder("Search controls…")
            .on_change(move |new: &str, _w, cx| {
                let e = entity_change.clone();
                e.update(cx, |s, _cx| s.nav_search = new.to_string());
            })
            .on_clear(move |_w, cx| {
                let e = entity_clear.clone();
                e.update(cx, |s, _cx| s.nav_search.clear());
            })
            .render(cx, window);

        // Back button — pops the page history.
        let can_back = !self.page_history.is_empty();
        let back_color = if can_back {
            secondary
        } else {
            theme_tertiary(cx)
        };
        let back = div()
            .id("tb-back")
            .size(px(32.0))
            .rounded(px(4.0))
            .flex()
            .items_center()
            .justify_center()
            .hover(move |s| s.bg(hover_bg))
            .child(
                icon("tb-back-ic", IconSource::Builtin("arrow-left".into()), cx)
                    .color(back_color)
                    .size(px(14.0))
                    .render(cx),
            )
            .on_click(move |_ev, _w, cx| {
                entity_back.update(cx, |s, _cx| {
                    if let Some(p) = s.page_history.pop() {
                        s.page = p;
                    }
                });
            });

        // Sidebar toggle — collapses / expands the navigation.
        let toggle = div()
            .id("tb-toggle")
            .size(px(32.0))
            .rounded(px(4.0))
            .flex()
            .items_center()
            .justify_center()
            .hover(move |s| s.bg(hover_bg))
            .child(
                label("tb-toggle-ic", "\u{2630}", cx)
                    .render(cx)
                    .text_size(px(16.0))
                    .text_color(secondary),
            )
            .on_click(move |_ev, _w, cx| {
                entity_toggle.update(cx, |s, _cx| s.sidebar_collapsed = !s.sidebar_collapsed);
            });

        div()
            .id("winui-titlebar")
            .w_full()
            .h(px(48.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(12.0))
            .child(back)
            .child(toggle)
            .child(div().w(px(8.0)))
            .child(brand_tile)
            .child(
                label("tb-brand", "WinUI", cx)
                    .render(cx)
                    .text_size(px(14.0)),
            )
            .child(div().flex_1())
            .child(div().w(px(320.0)).child(search))
            .into_any_element()
    }

    // -----------------------------------------------------------------
    // Sidebar
    // -----------------------------------------------------------------

    fn render_sidebar(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let nav_bg = {
            let t = cx.theme();
            // Sidebar background must match the title bar / app
            // background exactly (the WinUI Gallery keeps the nav
            // and title bar on the same Mica-tinted dark surface).
            t.get_color("winui.app_bg")
                .unwrap_or_else(|| t.get_color("surface.base").unwrap_or_default())
        };
        let accent = {
            let t = cx.theme();
            t.get_color("winui.accent")
                .unwrap_or_else(|| t.get_color("action.primary.bg").unwrap_or_default())
        };

        // The scrollable nav list (flex_1) and the footer live in
        // separate fixed slots of the sidebar shell, so the footer
        // can never be pushed below the viewport by tall content.
        let mut nav_list = div()
            .id("winui-nav-list")
            .relative()
            .w_full()
            .flex_1()
            .overflow_hidden()
            .py(px(16.0))
            .px(px(12.0))
            .flex()
            .flex_col()
            .gap(px(4.0));

        // Single animated selected rail (moves between items). Its
        // horizontal position tracks the ICON column (left = 12px nav
        // padding + item indent), so the bar hugs the left side of the
        // selected item's icon, and indented children sit to its right.
        let rail = div()
            .absolute()
            .left_0()
            .w(px(3.0))
            .h(px(18.0))
            .rounded(px(2.0))
            .bg(accent);
        let (rail_top, rail_left) = self.nav_rail_target();
        let rail_animated = AnimatedRail::new("winui-rail", rail_top, rail_left, rail);

        // Home item.
        nav_list = nav_list.child(
            self.build_nav_item(WinuiPage::Home, 12, cx)
                .into_any_element(),
        );

        // "Basic input" group.
        nav_list = nav_list.child(self.render_group_header("basic", "Basic input", cx));
        if self.groups_open.contains("basic") {
            for page in self.visible_group_pages() {
                let item_id = format!("winui-nav-{:?}-fade", page);
                let item: Stateful<Div> = self.build_nav_item(page, 24, cx);
                let anim = item.with_animation(
                    item_id,
                    gpui::Animation::new(Duration::from_millis(180))
                        .with_easing(|t: f32| 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t)),
                    |this, p| this.opacity(p),
                );
                nav_list = nav_list.child(anim);
            }
        }

        // "Settings" group (collapsed).
        nav_list = nav_list.child(self.render_group_header("settings", "Settings", cx));

        // Footer in its own fixed bottom slot (not absolute): it is
        // always visible, on every page, independent of item height.
        let footer = div()
            .id("winui-nav-footer")
            .h(px(36.0))
            .px(px(12.0))
            .flex()
            .items_center()
            .child(
                label(
                    "winui-nav-footer-lbl",
                    "WinUI renderer · v1.0.0-Insider",
                    cx,
                )
                .muted(true)
                .render(cx),
            );

        div()
            .id("winui-nav")
            .relative()
            .w(px(260.0))
            .h_full()
            .bg(nav_bg)
            .flex()
            .flex_col()
            .child(rail_animated)
            .child(nav_list)
            .child(footer)
            .into_any_element()
    }

    /// Group titles with WinUI *line* chevrons (SVG arrows, not the
    /// filled triangle glyphs). Clicking toggles the group.
    fn render_group_header(
        &self,
        key: &'static str,
        title: &str,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let entity = cx.entity().clone();
        let open = self.groups_open.contains(key);
        let hover_bg = {
            let t = cx.theme();
            t.get_color("winui.ctrl_fill_hover").unwrap_or_default()
        };

        div()
            .id(format!("winui-nav-group-{key}"))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .px(px(6.0))
            .h(px(32.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg))
            .child(
                icon(
                    format!("winui-nav-group-{key}-chev"),
                    IconSource::Builtin(if open { "arrow-down" } else { "arrow-right" }.into()),
                    cx,
                )
                .color({
                    let t = cx.theme();
                    t.get_color("content.secondary").unwrap_or_default()
                })
                .size(px(10.0))
                .render(cx),
            )
            .child(
                label(format!("winui-nav-group-{key}-lbl"), title, cx)
                    .muted(true)
                    .render(cx),
            )
            .on_click(move |_ev, _w, cx| {
                entity.update(cx, |s, _cx| {
                    if !s.groups_open.insert(key) {
                        s.groups_open.remove(key);
                    }
                });
            })
            .into_any_element()
    }

    /// One sidebar row (icon + label + selected rail positioning
    /// reserved; the rail itself is drawn separately).
    fn build_nav_item(
        &self,
        page: WinuiPage,
        indent: usize,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let entity = cx.entity().clone();
        let target = page;
        let selected = self.page == page;
        let (fg, accent, hover_bg, selected_bg) = {
            let t = cx.theme();
            (
                if selected {
                    t.get_color("content.primary").unwrap_or_default()
                } else {
                    t.get_color("content.secondary").unwrap_or_default()
                },
                t.get_color("winui.accent")
                    .unwrap_or_else(|| t.get_color("action.primary.bg").unwrap_or_default()),
                t.get_color("winui.ctrl_fill_hover").unwrap_or_default(),
                t.get_color("winui.subtle_fill_secondary")
                    .unwrap_or_default(),
            )
        };
        let _ = accent;

        let mut item = div()
            .id(format!("winui-nav-{:?}", page))
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(12.0))
            .pl(px(indent as f32))
            .pr(px(12.0))
            .h(px(40.0))
            .rounded(px(4.0))
            .cursor_pointer();

        if selected {
            item = item.bg(selected_bg).hover(move |s| s.bg(selected_bg));
        } else {
            item = item
                .bg(gpui::hsla(0., 0., 0., 0.))
                .hover(move |s| s.bg(hover_bg));
        }

        item.child(
            icon(
                format!("winui-nav-{:?}-ic", page),
                IconSource::Builtin(page.icon_name().into()),
                cx,
            )
            .color(fg)
            .size(px(16.0))
            .render(cx),
        )
        .child(label(format!("winui-nav-{:?}-lbl", page), page.label(), cx).render(cx))
        .on_click(move |_ev, _w, cx| {
            entity.update(cx, |s, _cx| {
                if s.page != target {
                    if s.page_history.last() != Some(&s.page) {
                        s.page_history.push(s.page);
                    }
                    s.page = target;
                }
            });
        })
    }

    /// The nav rows currently visible under the "Basic input" group
    /// (respecting the title-bar search filter).
    fn visible_group_pages(&self) -> Vec<WinuiPage> {
        [
            WinuiPage::Buttons,
            WinuiPage::Inputs,
            WinuiPage::Toggles,
            WinuiPage::Lists,
        ]
        .into_iter()
        .filter(|p| {
            self.nav_search.is_empty()
                || p.label()
                    .to_lowercase()
                    .contains(&self.nav_search.to_lowercase())
        })
        .collect()
    }

    /// (top, left) offsets of the selected rail, in navbar-shell
    /// coordinates. `top` is vertically centred on the selected row;
    /// `left` hugs the icon column = 12px nav padding + item indent,
    /// so the bar sits at the LEFT of each item's icon.
    fn nav_rail_target(&self) -> (f32, f32) {
        const TOP_PAD: f32 = 16.0;
        const ITEM_H: f32 = 40.0;
        const GROUP_H: f32 = 32.0;
        const GAP: f32 = 4.0;
        const NAV_PAD_X: f32 = 12.0;
        const ICON_LEFT_FROM_ITEM: f32 = 0.0; // icon starts at item's pl
        const GAP_BEFORE_ICON: f32 = 7.0; // breathing room rail → icon

        let rail_y = |item_top: f32| item_top + (ITEM_H - 18.0) / 2.0;
        let rail_left = |indent: f32| NAV_PAD_X + ICON_LEFT_FROM_ITEM + indent - GAP_BEFORE_ICON;

        if self.page == WinuiPage::Home {
            return (rail_y(TOP_PAD), rail_left(12.0));
        }
        let mut y = TOP_PAD + ITEM_H + GAP;
        y += GROUP_H + GAP;
        let pages = self.visible_group_pages();
        if !self.groups_open.contains("basic") {
            return (rail_y(TOP_PAD + ITEM_H + GAP), rail_left(12.0));
        }
        if let Some(idx) = pages.iter().position(|p| *p == self.page) {
            return (rail_y(y + (idx as f32) * (ITEM_H + GAP)), rail_left(24.0));
        }
        (rail_y(TOP_PAD + ITEM_H + GAP), rail_left(12.0))
    }
}

impl Render for WinuiApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_bg = {
            let t = cx.theme();
            t.get_color("winui.app_bg")
                .unwrap_or_else(|| t.get_color("surface.base").unwrap_or_default())
        };
        let stroke = {
            let t = cx.theme();
            t.get_color("winui.card_stroke")
                .unwrap_or_else(|| t.get_color("border.default").unwrap_or_default())
        };

        let titlebar = self.render_titlebar(window, cx);

        // Sidebar is always mounted; its width animates between 260
        // and 0 so collapse / expand is a smooth motion.
        let sidebar_any = self.render_sidebar(cx);
        let sidebar_bar = div().h_full().overflow_hidden().child(sidebar_any);
        let sidebar_target = if self.sidebar_collapsed { 0.0 } else { 260.0 };
        let sidebar_animated =
            AnimatedWidthElement::new("winui-sidebar-anim", sidebar_target, sidebar_bar);

        let content_scroll = div()
            .id("winui-content-scroll")
            .flex_1()
            .h_full()
            .overflow_y_scroll()
            .child(pages::build(self, window, cx));

        // The body region is ABSOLUTELY positioned to fill the window
        // below the title bar (top 49 → bottom / left → right). Its
        // height is fixed by the window, so the right-hand content can
        // NEVER push the sidebar (or its footer) below the viewport —
        // the two regions only share the row's horizontal space.
        let body = div()
            .id("winui-body")
            .absolute()
            .top(px(49.0))
            .left_0()
            .right_0()
            .bottom_0()
            .overflow_hidden()
            .flex()
            .flex_row()
            .child(sidebar_animated)
            .child(div().w(px(1.0)).h_full().bg(stroke))
            .child(content_scroll);

        div()
            .id("winui-app")
            .relative()
            .size_full()
            .bg(app_bg)
            .child(titlebar)
            .child(
                div()
                    .absolute()
                    .top(px(48.0))
                    .left_0()
                    .right_0()
                    .h(px(1.0))
                    .bg(stroke),
            )
            .child(body)
    }
}

/// Reads the theme tertiary color (used when back is disabled).
fn theme_tertiary(cx: &mut Context<WinuiApp>) -> gpui::Hsla {
    let t = cx.theme();
    t.get_color("content.tertiary")
        .unwrap_or_else(|| t.get_color("content.disabled").unwrap_or_default())
}

// ---------------------------------------------------------------------
// Animated selected rail
// ---------------------------------------------------------------------

#[derive(Clone)]
struct RailState {
    current_top: f32,
    current_left: f32,
}

/// A small custom element that eases a child's `top` AND `left`
/// toward target offsets every frame — used to animate the sidebar's
/// selected rail so it lands just left of each item's icon.
struct AnimatedRail {
    id: ElementId,
    target_top: f32,
    target_left: f32,
    child: Option<Div>,
}

impl AnimatedRail {
    fn new(id: impl Into<ElementId>, target_top: f32, target_left: f32, child: Div) -> Self {
        Self {
            id: id.into(),
            target_top,
            target_left,
            child: Some(child),
        }
    }
}

impl IntoElement for AnimatedRail {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for AnimatedRail {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let target_top = self.target_top;
        let target_left = self.target_left;
        let (top, left, animating) =
            window.with_element_state(global_id.unwrap(), |state: Option<RailState>, _window| {
                let mut st = state.unwrap_or(RailState {
                    current_top: target_top,
                    current_left: target_left,
                });
                let dt = target_top - st.current_top;
                let dl = target_left - st.current_left;
                let t_step = dt * 0.28;
                let l_step = dl * 0.28;
                st.current_top = if dt.abs() < 0.5 {
                    target_top
                } else {
                    st.current_top + t_step
                };
                st.current_left = if dl.abs() < 0.5 {
                    target_left
                } else {
                    st.current_left + l_step
                };
                (
                    (
                        st.current_top,
                        st.current_left,
                        (dt.abs() > 0.5 || dl.abs() > 0.5),
                    ),
                    st,
                )
            });
        if animating {
            window.request_animation_frame();
        }

        let child = self.child.take().expect("AnimatedRail used once");
        let mut element = child.top(px(top)).left(px(left)).into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

// ---------------------------------------------------------------------
// Animated sidebar collapse / expand
// ---------------------------------------------------------------------

#[derive(Clone)]
struct SidebarWidthState {
    current: f32,
}

/// Animates the sidebar container's width toward a target (260 →
/// 0 on collapse, and back), so collapsing/expanding the nav is a
/// smooth motion instead of a hard cut.
struct AnimatedWidthElement {
    id: ElementId,
    target: f32,
    child: Option<Div>,
}

impl AnimatedWidthElement {
    fn new(id: impl Into<ElementId>, target: f32, child: Div) -> Self {
        Self {
            id: id.into(),
            target,
            child: Some(child),
        }
    }
}

impl IntoElement for AnimatedWidthElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for AnimatedWidthElement {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let target = self.target;
        let (width, animating) = window.with_element_state(
            global_id.unwrap(),
            |state: Option<SidebarWidthState>, _window| {
                let mut st = state.unwrap_or(SidebarWidthState { current: target });
                let diff = target - st.current;
                let step = diff * 0.22;
                st.current = if diff.abs() < 1.0 {
                    target
                } else {
                    st.current + step
                };
                ((st.current, (st.current - target).abs() > 1.0), st)
            },
        );
        if animating {
            window.request_animation_frame();
        }

        let child = self.child.take().expect("AnimatedWidthElement used once");
        let mut element = child.w(px(width)).overflow_hidden().into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}
