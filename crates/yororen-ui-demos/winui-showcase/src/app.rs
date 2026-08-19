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
use yororen_ui::headless::modal::ModalState;
use yororen_ui::headless::popover::PopoverState;
use yororen_ui::headless::search_input::search_input;
use yororen_ui::headless::select::{SelectOption, SelectState};
use yororen_ui::headless::tree::TreeData;
use yororen_ui::headless::tree_item::TreeNodeId;
use yororen_ui::headless::virtual_list::{UniformVirtualListController, VirtualListController};
use yororen_ui::theme::ActiveTheme;

use crate::pages;

/// The pages reachable from the sidebar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WinuiPage {
    #[default]
    Home,
    Buttons,
    Inputs,
    Toggles,
    Lists,
    Status,
    Dialogs,
    Text,
    Data,
    Surfaces,
}

impl WinuiPage {
    pub fn label(self) -> &'static str {
        match self {
            WinuiPage::Home => "Home",
            WinuiPage::Buttons => "Buttons & Actions",
            WinuiPage::Inputs => "Inputs",
            WinuiPage::Toggles => "Toggles & Sliders",
            WinuiPage::Lists => "Lists & Menus",
            WinuiPage::Status => "Status & Feedback",
            WinuiPage::Dialogs => "Dialogs & Flyouts",
            WinuiPage::Text => "Text & Typography",
            WinuiPage::Data => "Data & Tables",
            WinuiPage::Surfaces => "Surfaces & Layout",
        }
    }

    pub fn title(self) -> &'static str {
        self.label()
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            WinuiPage::Home => "Bring WinUI experience to the web.",
            WinuiPage::Buttons => "Standard, primary, danger, icon, toggle and split buttons.",
            WinuiPage::Inputs => "Text, password, number, search, path and multiline inputs.",
            WinuiPage::Toggles => "Checkbox, switch, radio and slider — all live.",
            WinuiPage::Lists => "Select, combo box, listbox and dropdown menus.",
            WinuiPage::Status => {
                "Progress, skeleton, badges, tags, tooltip, empty state and avatars."
            }
            WinuiPage::Dialogs => "Modal dialog, popover and disclosure flyouts.",
            WinuiPage::Text => "Headings, labels, dividers and keyboard shortcuts.",
            WinuiPage::Data => "Table, tree, list items and virtualized lists.",
            WinuiPage::Surfaces => "Cards, panels, images, segmented groups and forms.",
        }
    }

    pub fn icon_name(self) -> &'static str {
        match self {
            WinuiPage::Home => "user",
            WinuiPage::Buttons => "check",
            WinuiPage::Inputs => "pencil",
            WinuiPage::Toggles => "arrow-left",
            WinuiPage::Lists => "folder",
            WinuiPage::Status => "info",
            WinuiPage::Dialogs => "window-close",
            WinuiPage::Text => "pencil",
            WinuiPage::Data => "arrow-down",
            WinuiPage::Surfaces => "maximize-on",
        }
    }
}

/// One sidebar group: a collapsible header plus its pages.
struct NavGroup {
    key: &'static str,
    title: &'static str,
    pages: &'static [WinuiPage],
}

/// The gallery navigation, mirroring the WinUI Gallery categories.
const NAV_GROUPS: &[NavGroup] = &[
    NavGroup {
        key: "basic",
        title: "Basic input",
        pages: &[WinuiPage::Buttons, WinuiPage::Inputs, WinuiPage::Toggles],
    },
    NavGroup {
        key: "collections",
        title: "Collections",
        pages: &[WinuiPage::Lists, WinuiPage::Data],
    },
    NavGroup {
        key: "dialogs",
        title: "Dialogs & flyouts",
        pages: &[WinuiPage::Dialogs],
    },
    NavGroup {
        key: "status",
        title: "Status & info",
        pages: &[WinuiPage::Status],
    },
    NavGroup {
        key: "text",
        title: "Text & layout",
        pages: &[WinuiPage::Text, WinuiPage::Surfaces],
    },
];

/// All interactive state for the demo app.
#[allow(dead_code)]
pub struct WinuiApp {
    // ---- navigation ----
    pub page: WinuiPage,
    pub page_history: Vec<WinuiPage>,
    pub sidebar_collapsed: bool,
    /// Live text of the title-bar search box (filters the sidebar).
    pub nav_search: String,
    /// Which sidebar groups are expanded.
    pub groups_open: std::collections::HashSet<&'static str>,

    // ---- composite `Entity<XxxState>` ----
    pub select_state: Entity<SelectState>,
    pub combo_state: Entity<ComboBoxState>,
    pub menu_state: Entity<MenuState>,
    pub dropdown_state: Entity<DropdownMenuState>,
    pub split_dd_state: Entity<DropdownMenuState>,
    pub listbox_state: Entity<ListboxState>,
    pub modal_state: Entity<ModalState>,
    pub popover_state: Entity<PopoverState>,
    pub tooltip_state: Entity<yororen_ui::headless::tooltip::TooltipState>,

    // ---- virtual lists ----
    pub vl_controller: VirtualListController,
    pub uvl_controller: UniformVirtualListController,

    // ---- tree ----
    pub tree_data: TreeData,
    pub tree_expanded: std::collections::BTreeSet<TreeNodeId>,
    pub tree_selected: Option<TreeNodeId>,

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

    pub progress: f32,
    pub disclosure_open: bool,
    pub tag_selected: bool,
    pub kbd: String,
    pub table_selected: usize,
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
        let modal_state = ModalState::new(&mut **cx);
        let popover_state = PopoverState::new(&mut **cx);
        let tooltip_state = yororen_ui::headless::tooltip::TooltipState::new(&mut **cx);

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
        modal_state.update(cx, |s, _cx| {
            s.set_dismiss_on_escape(true);
            s.set_dismiss_on_scrim(true);
            s.set_title("Confirm");
        });
        popover_state.update(cx, |s, _cx| {
            s.set_dismiss_on_escape(true);
            s.set_dismiss_on_outside_click(true);
        });

        let mut tree_data = TreeData::new();
        tree_data.add(
            None,
            yororen_ui::headless::tree::node_id("docs"),
            "Documents",
        );
        tree_data.add(
            Some(yororen_ui::headless::tree::node_id("docs")),
            yororen_ui::headless::tree::node_id("projects"),
            "Projects",
        );
        tree_data.add(
            Some(yororen_ui::headless::tree::node_id("docs")),
            yororen_ui::headless::tree::node_id("media"),
            "Media",
        );
        tree_data.add(
            Some(yororen_ui::headless::tree::node_id("projects")),
            yororen_ui::headless::tree::node_id("yororen"),
            "yororen-ui",
        );
        tree_data.add(
            Some(yororen_ui::headless::tree::node_id("projects")),
            yororen_ui::headless::tree::node_id("winui"),
            "winui-renderer",
        );
        tree_data.add(
            Some(yororen_ui::headless::tree::node_id("media")),
            yororen_ui::headless::tree::node_id("shots"),
            "screenshots",
        );
        let mut tree_expanded = std::collections::BTreeSet::new();
        tree_expanded.insert(yororen_ui::headless::tree::node_id("docs"));
        tree_expanded.insert(yororen_ui::headless::tree::node_id("projects"));

        let mut groups_open = std::collections::HashSet::new();
        groups_open.insert("basic");
        groups_open.insert("collections");
        groups_open.insert("dialogs");
        groups_open.insert("status");
        groups_open.insert("text");

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
            modal_state,
            popover_state,
            tooltip_state,
            vl_controller: VirtualListController::new(10_000, gpui::ListAlignment::Top, px(64.0)),
            uvl_controller: UniformVirtualListController::new(),
            tree_data,
            tree_expanded,
            tree_selected: None,
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
            progress: 0.45,
            disclosure_open: true,
            tag_selected: false,
            kbd: String::new(),
            table_selected: 0,
        }
    }

    // -----------------------------------------------------------------
    // Modal overlay
    // -----------------------------------------------------------------

    /// Scrim + centered dialog, deferred above all page content. The
    /// panel visuals (ContentDialog spec: 24px padding, 8px radius,
    /// deep shadow, fade + slide-up) belong to the modal renderer.
    fn build_modal_overlay(&self, cx: &mut Context<Self>) -> gpui::Deferred {
        use yororen_ui::headless::modal::modal;

        let is_visible = self.modal_state.read(cx).is_visible();
        if !is_visible {
            return gpui::deferred(div()).with_priority(2);
        }

        let modal_state_for_close = self.modal_state.clone();
        let modal_state_for_primary = self.modal_state.clone();
        let entity = cx.entity().clone();

        let panel = modal("winui-modal", self.modal_state.clone())
            .child(
                yororen_ui::headless::label::label("winui-modal-title", "Confirm changes", cx)
                    .strong(true)
                    .render(cx)
                    .text_size(px(20.0)),
            )
            .child(
                yororen_ui::headless::label::label(
                    "winui-modal-body",
                    "Discard unsaved changes and close the editor? This cannot be undone.",
                    cx,
                )
                .muted(true)
                .render(cx),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.0))
                    .child(
                        yororen_ui::headless::button::button("winui-modal-primary", cx)
                            .variant(yororen_ui::ActionVariantKind::Primary)
                            .on_click(move |_, _, cx| {
                                modal_state_for_primary.update(cx, |st, _cx| st.close());
                                entity.update(cx, |s, _cx| s.primary_clicks += 1);
                            })
                            .render(cx)
                            .child("Confirm"),
                    )
                    .child(
                        yororen_ui::headless::button::button("winui-modal-close", cx)
                            .on_click(move |_, _, cx| {
                                modal_state_for_close.update(cx, |st, _cx| st.close());
                            })
                            .render(cx)
                            .child("Cancel"),
                    ),
            )
            .render(cx)
            .w(px(360.0));

        let scrim = {
            let t = cx.theme();
            t.get_color("surface.scrim")
                .unwrap_or_else(|| gpui::hsla(0., 0., 0., 0.30))
        };

        // Scrim fades in with the dialog (reference ContentDialog
        // overlay opacity 83ms→250ms fast-out-slow-in family).
        let fade_ms = {
            let t = cx.theme();
            t.get_number("tokens.motion.duration_modal_fade")
                .unwrap_or(200.0) as u64
        };
        let scrim_inner = div()
            .absolute()
            .inset_0()
            .bg(scrim)
            .flex()
            .items_center()
            .justify_center()
            .child(panel);
        let scrim_el = yororen_ui_winui_renderer::animation::fade_in_on_mount(
            scrim_inner,
            "winui-modal-scrim-fade",
            std::time::Duration::from_millis(fade_ms),
            yororen_ui_winui_renderer::animation::fast_out_slow_in,
        );

        gpui::deferred(scrim_el).with_priority(2)
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
        // padding + item indent), so the bar hugs the left side of
        // the selected item's icon. When the selected page's row is
        // not visible (its group collapsed, or filtered out by the
        // search box) the rail is hidden entirely.
        let rail_el: gpui::AnyElement = match self.nav_rail_target() {
            Some((rail_top, rail_left)) => AnimatedRail::new(
                "winui-rail",
                rail_top,
                rail_left,
                div()
                    .absolute()
                    .left_0()
                    .w(px(3.0))
                    .h(px(18.0))
                    .rounded(px(2.0))
                    .bg(accent),
            )
            .into_any_element(),
            None => div().into_any_element(),
        };

        // Home item.
        nav_list = nav_list.child(
            self.build_nav_item(WinuiPage::Home, 12, cx)
                .into_any_element(),
        );

        // Collapsible groups (WinUI Gallery categories).
        for group in NAV_GROUPS {
            nav_list = nav_list.child(self.render_group_header(group.key, group.title, cx));
            if self.groups_open.contains(group.key) {
                for page in self.visible_group_pages(group) {
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
        }

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
            .child(rail_el)
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

    /// The nav rows currently visible under `group` (respecting the
    /// title-bar search filter).
    fn visible_group_pages(&self, group: &NavGroup) -> Vec<WinuiPage> {
        group
            .pages
            .iter()
            .copied()
            .filter(|p| {
                self.nav_search.is_empty()
                    || p.label()
                        .to_lowercase()
                        .contains(&self.nav_search.to_lowercase())
            })
            .collect()
    }

    /// (top, left) offsets of the selected rail, in navbar-shell
    /// coordinates. Walks the same static group structure the
    /// renderer uses, accumulating row heights: the rail lands
    /// vertically centred on the selected row, hugging the icon
    /// column.
    /// `(top, left)` offsets of the selected rail, or `None` when
    /// the selected row is not currently visible (its group is
    /// collapsed, or the title-bar search filtered it out) — in
    /// which case the rail is hidden rather than parked somewhere
    /// misleading.
    fn nav_rail_target(&self) -> Option<(f32, f32)> {
        const TOP_PAD: f32 = 16.0;
        const ITEM_H: f32 = 40.0;
        const GROUP_H: f32 = 32.0;
        const GAP: f32 = 4.0;
        const NAV_PAD_X: f32 = 12.0;
        const GAP_BEFORE_ICON: f32 = 10.0;

        let rail_y = |item_top: f32| item_top + (ITEM_H - 18.0) / 2.0;
        let rail_left = |indent: f32| NAV_PAD_X + indent - GAP_BEFORE_ICON;

        if self.page == WinuiPage::Home {
            return Some((rail_y(TOP_PAD), rail_left(12.0)));
        }

        let mut y = TOP_PAD + ITEM_H + GAP;
        for group in NAV_GROUPS {
            y += GROUP_H + GAP;
            if !self.groups_open.contains(group.key) {
                continue;
            }
            let pages = self.visible_group_pages(group);
            if let Some(idx) = pages.iter().position(|p| *p == self.page) {
                return Some((rail_y(y + (idx as f32) * (ITEM_H + GAP)), rail_left(24.0)));
            }
            y += pages.len() as f32 * (ITEM_H + GAP);
        }

        // The selected page's group is collapsed or the row is
        // filtered out — nothing visible to point at.
        None
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

        // Global modal overlay: scrim + centered dialog, painted via
        // `deferred` above the page content (priority 2 keeps it
        // below toasts and above popover/dropdown panels).
        let modal_layer = self.build_modal_overlay(cx);

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
            .child(modal_layer)
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
