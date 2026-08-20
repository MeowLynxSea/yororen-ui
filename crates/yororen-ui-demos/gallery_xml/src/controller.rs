//! Controller — every business-logic method the XML
//! references. The XML itself stays purely declarative:
//! every `on_click={controller.foo}` resolves to a method
//! here via the macro's auto-wrap.
//!
//! `Controller` is `Clone` because the macro pre-clones the
//! receiver into a hygienic local per event handler.

use std::collections::{BTreeSet, HashMap};
use std::ops::Range;

use gpui::{App, ClickEvent, Entity, IntoElement, SharedString, Window};

use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
use yororen_ui::headless::keybinding_input::KeybindingInputMode;
use yororen_ui::headless::table::TableColumn;
use yororen_ui::headless::tree::TreeData;
use yororen_ui::headless::tree_item::TreeNodeId;
use yororen_ui::notification::center::{Notification, NotificationCenter, ToastKind};

use crate::state::{DarkMode, GalleryState, LocaleChoice, RendererKind};
use yororen_ui::i18n::Translate;
use yororen_ui::t_named;

#[derive(Clone)]
pub struct Controller {
    state: Entity<GalleryState>,
}

// Thin 4-arg event adapters for the toolbar toggle buttons so the
// XML can use bare method references instead of inline closures.
macro_rules! toggle_selectors {
    ($($name:ident => $method:ident($value:expr)),* $(,)?) => {
        $(
            pub fn $name(
                &self,
                _checked: bool,
                _ev: Option<&ClickEvent>,
                _w: &mut Window,
                cx: &mut App,
            ) {
                self.$method($value, _w, cx);
            }
        )*
    };
}

// Thin 4-arg event adapters so the XML can use bare method
// references (`on_toggle={controller.select_radio_0}`) instead
// of inline Rust closures.
macro_rules! radio_selectors {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(
            pub fn $name(
                &self,
                _checked: bool,
                _ev: Option<&ClickEvent>,
                _w: &mut Window,
                cx: &mut App,
            ) {
                self.set_radio($value, _w, cx);
            }
        )*
    };
}

impl Controller {
    pub fn new(state: Entity<GalleryState>, cx: &mut App) -> Self {
        Self::wire_composite_state(&state, cx);
        Self { state }
    }

    // -------- Field accessors (replaced the per-frame `snapshot()`) --------
    //
    // Each accessor does exactly one `state.read(cx)` and returns
    // either a `Copy` value (no allocation) or a `SharedString`
    // (single `Arc::clone`, no `String` clone). The XML layer
    // consumes these directly — `controller.snapshot(cx).foo`
    // is gone everywhere.

    // ---- Toolbar (Copy) ----
    pub fn current_renderer(&self, cx: &App) -> RendererKind {
        self.state.read(cx).current_renderer
    }
    pub fn dark_mode(&self, cx: &App) -> DarkMode {
        self.state.read(cx).dark_mode
    }
    pub fn current_locale(&self, cx: &App) -> LocaleChoice {
        self.state.read(cx).current_locale
    }
    pub fn toast_count(&self, cx: &App) -> usize {
        self.state.read(cx).toast_count.value
    }

    // ---- Display (Copy + usize) ----
    pub fn tag_selected(&self, cx: &App) -> bool {
        self.state.read(cx).tag_selected
    }
    pub fn tag_close_count(&self, cx: &App) -> usize {
        self.state.read(cx).tag_closable_count.value
    }
    pub fn progress_value(&self, cx: &App) -> f32 {
        self.state.read(cx).progress_value
    }
    pub fn toggle_btn_selected(&self, cx: &App) -> bool {
        self.state.read(cx).toggle_btn_selected
    }

    // ---- Overlays (Copy + SharedString) ----
    pub fn disclosure_open(&self, cx: &App) -> bool {
        self.state.read(cx).disclosure_open
    }
    pub fn popover_visible(&self, cx: &App) -> bool {
        self.state.read(cx).popover_state.read(cx).is_visible()
    }
    pub fn is_modal_open(&self, cx: &App) -> bool {
        self.state.read(cx).modal_state.read(cx).open
    }
    pub fn dropdown_demo_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).dropdown_demo_value.clone().into()
    }
    pub fn menu_demo_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).menu_demo_value.clone().into()
    }

    // ---- Lists (Copy + SharedString) ----
    pub fn selected_list_item(&self, cx: &App) -> Option<usize> {
        self.state.read(cx).selected_list_item
    }
    pub fn selected_table_row(&self, cx: &App) -> Option<usize> {
        self.state.read(cx).selected_table_row
    }
    pub fn form_submit_count(&self, cx: &App) -> usize {
        self.state.read(cx).form_submit_count
    }
    pub fn form_email_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).form_email_value.read(cx).clone().into()
    }
    pub fn form_email_error(&self, cx: &App) -> Option<String> {
        self.state.read(cx).form_email_error.clone()
    }

    // ---- Controls (Copy) ----
    pub fn checkbox_value(&self, cx: &App) -> bool {
        *self.state.read(cx).checkbox_value.read(cx)
    }
    pub fn switch_value(&self, cx: &App) -> bool {
        *self.state.read(cx).switch_value.read(cx)
    }
    pub fn radio_value(&self, cx: &App) -> usize {
        self.state.read(cx).radio_value
    }
    pub fn slider_value(&self, cx: &App) -> f32 {
        *self.state.read(cx).slider_value.read(cx)
    }

    // ---- Inputs (SharedString / Copy) ----
    pub fn text_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).text_value.read(cx).clone().into()
    }
    pub fn password_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).password_value.read(cx).clone().into()
    }
    pub fn number_value(&self, cx: &App) -> f64 {
        *self.state.read(cx).number_value.read(cx)
    }
    pub fn search_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).search_value.read(cx).clone().into()
    }
    pub fn file_path_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).file_path_value.read(cx).clone().into()
    }
    pub fn keybinding_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).keybinding_value.clone().into()
    }
    pub fn keybinding_mode(&self, cx: &App) -> KeybindingInputMode {
        self.state.read(cx).keybinding_mode
    }
    pub fn text_area_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).text_area_value.read(cx).clone().into()
    }
    pub fn select_demo_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).select_demo_value.clone().into()
    }
    pub fn combo_demo_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).combo_demo_value.clone().into()
    }

    pub fn editable_combo_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).editable_combo_value.clone().into()
    }
    pub fn listbox_demo_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).listbox_demo_value.clone().into()
    }

    pub fn gridview_demo_value(&self, cx: &App) -> SharedString {
        self.state.read(cx).gridview_demo_value.clone().into()
    }

    // -------- Footer text helpers (used by `<Label text={controller.footer_X_text(cx)}>`) --------

    /// Build the footer form-summary text in a single state read.
    pub fn footer_form_text(&self, cx: &App) -> String {
        t_named!(cx, "demo.footer.form_summary",
            count => self.form_submit_count(cx),
            email => self.form_email_value(cx),
            error => format!("{:?}", self.form_email_error(cx)))
        .to_string()
    }

    /// Build the footer controls-summary text in a single state read.
    pub fn footer_controls_text(&self, cx: &App) -> String {
        t_named!(cx, "demo.footer.controls_summary",
            checkbox => self.checkbox_value(cx),
            switch => self.switch_value(cx),
            radio => self.radio_value(cx),
            slider => format!("{:.1}", self.slider_value(cx)))
        .to_string()
    }

    /// Build the footer toast-summary text in a single state read.
    pub fn footer_toast_text(&self, cx: &App) -> String {
        t_named!(cx, "demo.footer.toast_summary",
            count => self.toast_count(cx),
            locale => self.current_locale(cx).tag())
        .to_string()
    }

    // -------- Entity accessors used by `@bind` and view.rs --------

    pub fn modal_state(&self, cx: &App) -> Entity<yororen_ui::headless::modal::ModalState> {
        self.state.read(cx).modal_state.clone()
    }

    // -------- Composite state accessors --------

    pub fn select_state(&self, cx: &App) -> Entity<yororen_ui::headless::select::SelectState> {
        self.state.read(cx).select_state.clone()
    }

    pub fn combo_state(&self, cx: &App) -> Entity<yororen_ui::headless::combo_box::ComboBoxState> {
        self.state.read(cx).combo_state.clone()
    }

    pub fn editable_combo_state(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::combo_box::ComboBoxState> {
        self.state.read(cx).editable_combo_state.clone()
    }

    pub fn listbox_state(&self, cx: &App) -> Entity<yororen_ui::headless::listbox::ListboxState> {
        self.state.read(cx).listbox_state.clone()
    }

    pub fn gridview_state(&self, cx: &App) -> Entity<yororen_ui::headless::grid_view::GridViewState> {
        self.state.read(cx).gridview_state.clone()
    }

    pub fn menu_state(&self, cx: &App) -> Entity<yororen_ui::headless::menu::MenuState> {
        self.state.read(cx).menu_state.clone()
    }

    /// Body state for the `<Menu slot="content">` inside the
    /// dropdown demo. Items mirror `menu_state` but its
    /// `on_select` writes `dropdown_demo_value` and closes the
    /// dropdown, so the popover and dropdown summaries stay
    /// independent.
    pub fn dropdown_menu_state(&self, cx: &App) -> Entity<yororen_ui::headless::menu::MenuState> {
        self.state.read(cx).dropdown_menu_state.clone()
    }

    pub fn dropdown_state(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::dropdown_menu::DropdownMenuState> {
        self.state.read(cx).dropdown_state.clone()
    }

    pub fn split_button_state(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::dropdown_menu::DropdownMenuState> {
        self.state.read(cx).split_dropdown_state.clone()
    }

    /// Open/close state for the toggle split_button demo cell —
    /// deliberately a separate `DropdownMenuState` so the two
    /// split buttons open/close independently.
    pub fn toggle_split_button_state(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::dropdown_menu::DropdownMenuState> {
        self.state.read(cx).split_toggle_dropdown_state.clone()
    }

    pub fn popover_state(&self, cx: &App) -> Entity<yororen_ui::headless::popover::PopoverState> {
        self.state.read(cx).popover_state.clone()
    }

    pub fn tooltip_state(&self, cx: &App) -> Entity<yororen_ui::headless::tooltip::TooltipState> {
        self.state.read(cx).tooltip_state.clone()
    }

    pub fn list_controller_entity(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::virtual_list::VirtualListController> {
        self.state.read(cx).list_controller.clone()
    }

    pub fn uniform_list_controller_entity(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::virtual_list::UniformVirtualListController> {
        self.state.read(cx).uniform_list_controller.clone()
    }

    pub fn vl_item_count(&self, cx: &App) -> usize {
        self.state.read(cx).vl_item_count
    }

    // -------- Entity accessors for `@bind` --------

    pub fn form_email_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).form_email_value.clone()
    }

    pub fn checkbox_value_entity(&self, cx: &App) -> Entity<bool> {
        self.state.read(cx).checkbox_value.clone()
    }

    pub fn switch_value_entity(&self, cx: &App) -> Entity<bool> {
        self.state.read(cx).switch_value.clone()
    }

    pub fn slider_value_entity(&self, cx: &App) -> Entity<f32> {
        self.state.read(cx).slider_value.clone()
    }

    pub fn text_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).text_value.clone()
    }

    pub fn password_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).password_value.clone()
    }

    pub fn number_value_entity(&self, cx: &App) -> Entity<f64> {
        self.state.read(cx).number_value.clone()
    }

    pub fn search_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).search_value.clone()
    }

    pub fn file_path_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).file_path_value.clone()
    }

    pub fn text_area_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).text_area_value.clone()
    }

    // -------- Collection / complex accessors --------

    pub fn list_items(&self, cx: &App) -> Vec<gpui::SharedString> {
        self.state.read(cx).list_items.clone()
    }

    pub fn tree_expanded(&self, cx: &App) -> BTreeSet<TreeNodeId> {
        self.state.read(cx).tree_expanded.clone()
    }

    pub fn tree_selected(&self, cx: &App) -> Option<TreeNodeId> {
        self.state.read(cx).tree_selected.clone()
    }

    // -------- Toolbar actions --------

    pub fn set_renderer(&self, kind: RendererKind, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| s.current_renderer = kind);
    }

    pub fn set_dark_mode(&self, dark: DarkMode, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| s.dark_mode = dark);
    }

    pub fn set_locale(&self, choice: LocaleChoice, _w: &mut Window, cx: &mut App) {
        crate::i18n::install_for_locale(cx, choice);
        self.state.update(cx, |s, cx| {
            s.current_locale = choice;
            // Re-resolve translated labels so the tree stays in
            // sync with the new locale.
            s.tree_data = GalleryState::build_tree_data(cx);
        });
    }

    toggle_selectors! {
        select_default_renderer => set_renderer(RendererKind::Default),
        select_brutalism_renderer => set_renderer(RendererKind::Brutalism),
        select_light_theme => set_dark_mode(DarkMode::Light),
        select_dark_theme => set_dark_mode(DarkMode::Dark),
        select_english_locale => set_locale(LocaleChoice::En),
        select_chinese_locale => set_locale(LocaleChoice::ZhCn),
        select_arabic_locale => set_locale(LocaleChoice::Ar),
    }

    pub fn show_toast(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        let id = self.state.update(cx, |s, _cx| {
            s.toast_count.value += 1;
            s.toast_count.value
        });
        let id_str = id.to_string();
        let center = cx.global::<NotificationCenter>().clone();
        center.notify(
            Notification::new(cx.t_with_args("demo.toast_message", &[&id_str]))
                .title(cx.t("demo.toast_title"))
                .kind(ToastKind::Info),
            cx,
        );
    }

    pub fn show_notification(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        let id = self.state.update(cx, |s, _cx| {
            s.toast_count.value += 1;
            s.toast_count.value
        });
        let id_str = id.to_string();
        let center = cx.global::<NotificationCenter>().clone();
        center.notify(
            Notification::new(cx.t_with_args("demo.notification_message", &[&id_str]))
                .title(cx.t("demo.notification_title"))
                .kind(ToastKind::Success)
                .sticky(true),
            cx,
        );
    }

    // -------- Actions section --------

    pub fn split_button_items(&self, _cx: &App) -> Vec<DropdownItem> {
        vec![
            DropdownItem::Item(DropdownMenuItem::new("save", _cx.t("demo.actions.save"))),
            DropdownItem::Item(DropdownMenuItem::new(
                "save_as",
                _cx.t("demo.actions.save_as"),
            )),
            DropdownItem::Item(DropdownMenuItem::new(
                "save_all",
                _cx.t("demo.actions.save_all"),
            )),
        ]
    }

    pub fn split_button_primary(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.toast_count.value += 1;
            // gpui does not repaint on `Entity::update` by
            // itself — notify or the click appears lost.
            cx.notify();
        });
    }

    // -------- Toggle split_button (WinUI ToggleSplitButton) ----

    pub fn split_button_toggled(&self, cx: &App) -> bool {
        self.state.read(cx).split_toggle_on
    }

    /// Flyout item currently chosen on the toggle split_button;
    /// its label replaces the primary half's caption.
    pub fn split_button_selected_item(&self, cx: &App) -> Option<SharedString> {
        self.state.read(cx).split_toggle_selected.clone()
    }

    pub fn toggle_split_button_items(&self, _cx: &App) -> Vec<DropdownItem> {
        vec![
            DropdownItem::Item(DropdownMenuItem::new(
                "mute",
                _cx.t("demo.actions.mute_mic"),
            )),
            DropdownItem::Item(DropdownMenuItem::new(
                "mute_speaker",
                _cx.t("demo.actions.mute_speaker"),
            )),
            DropdownItem::Item(DropdownMenuItem::new(
                "unmute",
                _cx.t("demo.actions.unmute"),
            )),
        ]
    }

    /// The primary half of a toggle split button flips the
    /// caller-owned checked bit; the renderer paints the accent
    /// fill while it is set.
    pub fn toggle_split_button_primary(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.split_toggle_on = !s.split_toggle_on;
            cx.notify();
        });
    }

    /// Picking a flyout option makes it the primary half's
    /// caption AND turns the toggle on (the bullet-list
    /// ToggleSplitButton pattern).
    pub fn toggle_split_button_select(
        &self,
        id: gpui::SharedString,
        _w: &mut Window,
        cx: &mut App,
    ) {
        self.state.update(cx, |s, cx| {
            s.split_toggle_selected = Some(id);
            s.split_toggle_on = true;
            cx.notify();
        });
    }

    pub fn split_button_select(&self, _id: gpui::SharedString, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.toast_count.value += 1;
            cx.notify();
        });
    }

    pub fn noop_click(&self, _ev: &ClickEvent, _w: &mut Window, _cx: &mut App) {}

    /// Adapter for `<ToggleButton on_toggle={controller.press_toggle_from_event}>`.
    /// The XML macro auto-wraps this 4-arg event signature into the
    /// 2-arg business-logic method.
    pub fn press_toggle_from_event(
        &self,
        selected: bool,
        _ev: Option<&ClickEvent>,
        _w: &mut Window,
        cx: &mut App,
    ) {
        self.state.update(cx, |s, _cx| {
            s.toggle_btn_selected = selected;
        });
    }

    pub fn close_tag(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.tag_closable_count.value += 1;
        });
    }

    pub fn toggle_tag_selected(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.tag_selected = !s.tag_selected;
        });
    }

    // -------- Controls section --------
    //
    // `checkbox_value` / `switch_value` / `slider_value` are
    // bound through `@bind` in the XML — the renderer writes
    // into the entity directly. No controller-side setter is
    // needed for the demo.

    pub fn set_radio(&self, value: usize, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.radio_value = value;
        });
    }

    radio_selectors! {
        select_radio_0 => 0,
        select_radio_1 => 1,
        select_radio_2 => 2,
    }

    // -------- Inputs section --------
    //
    // `text_value` / `password_value` / `search_value` /
    // `file_path_value` / `text_area_value` are bound through
    // `@bind` in the XML — the renderer writes into the entity
    // directly and the `@bind` expression fires `on_change`
    // for the demo. No controller-side setter is needed.

    pub fn clear_search(&self, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).search_value.clone();
        entity.update(cx, |s, _cx| s.clear());
    }

    pub fn set_number_value(&self, value: f64, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).number_value.clone();
        entity.update(cx, |s, _cx| *s = value);
    }

    pub fn set_keybinding_value(&self, value: &str, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.keybinding_value = value.to_string();
        });
    }

    pub fn browse_file_path(&self, _value: &str, _w: &mut Window, _cx: &mut App) {
        // The renderer already wrote the picked path into the
        // input's bound state and fired `on_change`; this hook is
        // only for extra work (logging, validation, etc.).
    }

    pub fn start_keybinding_capture(&self, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.keybinding_mode =
                yororen_ui::headless::keybinding_input::KeybindingInputMode::Capturing;
        });
    }

    pub fn cancel_keybinding_capture(&self, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.keybinding_mode = yororen_ui::headless::keybinding_input::KeybindingInputMode::Idle;
        });
    }

    // -------- Overlays section --------

    pub fn open_modal(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, ctx| {
            s.modal_state.update(ctx, |st, _| st.open());
        });
    }

    pub fn close_modal(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, ctx| {
            s.modal_state.update(ctx, |st, _| st.close());
        });
    }

    pub fn toggle_popover(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, ctx| {
            s.popover_state.update(ctx, |st, _| st.toggle());
        });
    }

    pub fn toggle_dropdown(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, ctx| {
            s.dropdown_state.update(ctx, |st, _| st.toggle());
        });
    }

    pub fn toggle_disclosure(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.disclosure_open = !s.disclosure_open;
        });
    }

    // -------- Lists section --------

    pub fn submit_form(&self, vals: HashMap<SharedString, String>, _w: &mut Window, cx: &mut App) {
        // Read the `email` value out of the form's submitted map
        // (set by the `<FormField name="email">` child). Falls
        // back to the currently bound string when the field is
        // missing — keeps the validation error path identical
        // to before without re-reading the entity.
        let email = vals
            .get(&SharedString::from("email"))
            .cloned()
            .unwrap_or_else(|| self.form_email_value(cx).to_string());
        let must_contain = cx.t("demo.form.must_contain_at");
        self.state.update(cx, |s, _cx| {
            s.form_submit_count += 1;
            s.form_email_error = if email.contains('@') {
                None
            } else {
                Some(must_contain.to_string())
            };
        });
    }

    pub fn table_row_handler(
        &self,
    ) -> impl Fn(usize, &mut gpui::Window, &mut gpui::App) + Clone + 'static {
        let state = self.state.clone();
        move |value, _w, cx| {
            state.update(cx, |s, _cx| {
                s.selected_table_row = Some(value);
            });
        }
    }

    pub fn select_list_item_handler(
        &self,
        index: usize,
    ) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, _cx| {
                s.selected_list_item = Some(index);
            });
        }
    }

    pub fn virtual_list_scroll_top(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.list_controller_entity(cx)
            .update(cx, |c, _| c.scroll_to_top());
    }

    pub fn virtual_list_scroll_bottom(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.list_controller_entity(cx)
            .update(cx, |c, _| c.scroll_to_bottom());
    }

    pub fn uniform_list_scroll_top(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.uniform_list_controller_entity(cx)
            .update(cx, |c, _| c.scroll_to_top());
    }

    pub fn uniform_list_scroll_bottom(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.uniform_list_controller_entity(cx)
            .update(cx, |c, _| c.scroll_to_bottom());
    }

    pub fn on_visible_range_change(
        &self,
        range: Range<usize>,
        total: usize,
        _w: &mut Window,
        cx: &mut App,
    ) {
        self.state.update(cx, |s, _cx| {
            s.vl_visible_range = Some(range.clone());
            // Only load the next batch when the visible end is
            // within 50 items of the total (matches gallery_demo).
            if range.end + 50 >= total && s.vl_item_count < 5_000 {
                s.vl_item_count += 100;
                s.vl_load_count += 1;
            }
        });
    }

    // -------- Complex layout helpers (used by lists.xml) --------

    pub fn listbox_status_text(&self, cx: &App) -> String {
        let value = self.listbox_demo_value(cx);
        let status = if value.is_empty() {
            "—".to_string()
        } else {
            value.to_string()
        };
        cx.t("demo.lists.listbox_selected")
            .replacen("{}", &status, 1)
    }

    pub fn gridview_status_text(&self, cx: &App) -> String {
        let value = self.gridview_demo_value(cx);
        let status = if value.is_empty() {
            "—".to_string()
        } else {
            value.to_string()
        };
        cx.t("demo.lists.gridview_selected")
            .replacen("{}", &status, 1)
    }

    pub fn form_email_error_text(&self, cx: &App) -> String {
        self.form_email_error(cx).unwrap_or_default()
    }

    pub fn form_status_text(&self, cx: &App) -> String {
        format!(
            "{} {} | {} {:?}",
            cx.t("demo.form.submitted"),
            self.form_submit_count(cx),
            cx.t("demo.form.last_error"),
            self.form_email_error(cx)
        )
    }

    pub fn tree_label(&self, id: &TreeNodeId, cx: &App) -> String {
        let state = self.state.read(cx);
        state.tree_data.label_of(id).unwrap_or("").to_string()
    }

    pub fn tree_has_children(&self, id: &TreeNodeId, cx: &App) -> bool {
        let state = self.state.read(cx);
        !state.tree_data.children_of(id).is_empty()
    }

    pub fn toggle_tree_node_for(
        &self,
        id: TreeNodeId,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, cx| {
                if !s.tree_expanded.remove(&id) {
                    s.tree_expanded.insert(id.clone());
                }
                cx.notify();
            });
        }
    }

    pub fn select_tree_node_for(
        &self,
        id: TreeNodeId,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, cx| {
                s.tree_selected = Some(id.clone());
                cx.notify();
            });
        }
    }

    // -------- Multi-select tree helpers --------

    pub fn tree_multi_selected(&self, cx: &App) -> BTreeSet<TreeNodeId> {
        self.state.read(cx).tree_multi_selected.clone()
    }

    /// Expansion set for the multi-select tree — separate from
    /// the single-select tree's so the two cells expand
    /// independently.
    pub fn tree_multi_expanded(&self, cx: &App) -> BTreeSet<TreeNodeId> {
        self.state.read(cx).tree_multi_expanded.clone()
    }

    pub fn tree_multi_checked(&self, id: &TreeNodeId, cx: &App) -> bool {
        self.state.read(cx).tree_multi_selected.contains(id)
    }

    /// Parent rows show the mixed dash while some (but not all)
    /// descendants are checked.
    pub fn tree_multi_indeterminate(&self, id: &TreeNodeId, cx: &App) -> bool {
        let state = self.state.read(cx);
        if state.tree_data.children_of(id).is_empty() {
            return false;
        }
        let mut descendants = state
            .tree_data
            .subtree_ids(id)
            .into_iter()
            .filter(|n| n != id);
        let some = descendants
            .clone()
            .any(|n| state.tree_multi_selected.contains(&n));
        let all = descendants.all(|n| state.tree_multi_selected.contains(&n));
        some && !all
    }

    /// Expansion toggler for the multi-select tree (its own
    /// expansion set — see `tree_multi_expanded`).
    pub fn toggle_tree_multi_node_for(
        &self,
        id: TreeNodeId,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, cx| {
                if !s.tree_multi_expanded.remove(&id) {
                    s.tree_multi_expanded.insert(id.clone());
                }
                cx.notify();
            });
        }
    }

    /// `on_check` for a multi-select row: toggles the node's
    /// whole subtree (`TreeData::subtree_ids`).
    pub fn toggle_tree_subtree_for(
        &self,
        id: TreeNodeId,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, cx| {
                let subtree = s.tree_data.subtree_ids(&id);
                if subtree.iter().all(|n| s.tree_multi_selected.contains(n)) {
                    for n in subtree {
                        s.tree_multi_selected.remove(&n);
                    }
                } else {
                    for n in subtree {
                        s.tree_multi_selected.insert(n);
                    }
                }
                cx.notify();
            });
        }
    }

    /// `on_click` for a multi-select row: plain click selects
    /// just the node, ctrl/cmd-click toggles it, shift-click
    /// selects the visible range from the anchor.
    pub fn select_tree_multi_for(
        &self,
        id: TreeNodeId,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |ev: &ClickEvent, _w, cx| {
            let mods = match ev {
                gpui::ClickEvent::Mouse(m) => m.down.modifiers,
                _ => gpui::Modifiers::default(),
            };
            state.update(cx, |s, cx| {
                if mods.shift
                    && let Some(anchor) = s.tree_anchor.clone()
                {
                    s.tree_multi_selected =
                        s.tree_data
                            .select_range(&s.tree_multi_expanded, &anchor, &id);
                } else if mods.control || mods.platform {
                    if !s.tree_multi_selected.remove(&id) {
                        s.tree_multi_selected.insert(id.clone());
                    }
                } else {
                    s.tree_multi_selected = BTreeSet::from([id.clone()]);
                }
                s.tree_anchor = Some(id.clone());
                cx.notify();
            });
        }
    }

    pub fn vl_status_text(&self, cx: &App) -> String {
        let s = self.state.read(cx);
        let visible = format!("{:?}", s.vl_visible_range);
        cx.t("demo.lists.vl_status")
            .replacen("{:?}", &visible, 1)
            .replacen("{}", &s.vl_item_count.to_string(), 1)
            .replacen("{}", &s.vl_load_count.to_string(), 1)
    }

    pub fn virtual_list_row(
        &self,
        cx: &App,
    ) -> impl FnMut(usize, &mut Window, &mut App) -> gpui::AnyElement + 'static {
        let state = self.state.clone();
        let item_template = cx.t("demo.lists.vl_item");
        move |ix, _w, cx| {
            let selected = state.read(cx).selected_list_item == Some(ix);
            let row_id: gpui::ElementId = format!("vl-row-{ix}").into();
            let row_label = item_template.replacen("{}", &ix.to_string(), 1);
            let state_for_click = state.clone();
            yororen_ui::headless::list_item::list_item(row_id, row_label, cx)
                .selected(selected)
                .on_click(move |_ev, _w, cx| {
                    state_for_click.update(cx, |s, _cx| {
                        s.selected_list_item = Some(ix);
                    });
                })
                .render(cx)
                .into_any_element()
        }
    }

    pub fn uniform_list_row(
        &self,
        cx: &App,
    ) -> impl FnMut(usize, &mut Window, &mut App) -> gpui::AnyElement + 'static {
        let item_template = cx.t("demo.lists.uvl_item");
        move |ix, _w, cx| {
            let row_id: gpui::ElementId = format!("uvl-row-{ix}").into();
            let row_label = item_template.replacen("{}", &ix.to_string(), 1);
            yororen_ui::headless::list_item::list_item(row_id, row_label, cx)
                .render(cx)
                .into_any_element()
        }
    }

    // -------- Data helpers used by XML --------

    pub fn table_columns(&self, cx: &App) -> Vec<TableColumn> {
        vec![
            TableColumn::new("name", cx.t("demo.lists.table_col_name")).width(120.),
            TableColumn::new("age", cx.t("demo.lists.table_col_age")).width(60.),
            TableColumn::new("city", cx.t("demo.lists.table_col_city")).width(120.),
        ]
    }

    pub fn table_rows(&self, cx: &App) -> Vec<Vec<SharedString>> {
        vec![
            vec![
                cx.t("demo.lists.table_row_alice"),
                cx.t("demo.lists.table_row_age_30"),
                cx.t("demo.lists.table_row_beijing"),
            ],
            vec![
                cx.t("demo.lists.table_row_bob"),
                cx.t("demo.lists.table_row_age_25"),
                cx.t("demo.lists.table_row_shanghai"),
            ],
            vec![
                cx.t("demo.lists.table_row_carol"),
                cx.t("demo.lists.table_row_age_28"),
                cx.t("demo.lists.table_row_shenzhen"),
            ],
        ]
    }

    pub fn tree_data(&self, cx: &App) -> TreeData {
        // Tree data is built once in `GalleryState::new_data` and
        // refreshed in `set_locale`. Cloning the cached value is
        // cheaper than rebuilding the maps and resolving translations
        // on every frame.
        self.state.read(cx).tree_data.clone()
    }

    // -------- Internal wiring --------

    fn wire_composite_state(state: &Entity<GalleryState>, cx: &mut App) {
        state.update(cx, |s, cx| {
            let state_for_select = state.clone();
            s.select_state.update(cx, |st, _cx| {
                st.set_on_change(move |value, _w, cx| {
                    let v = value.to_string();
                    state_for_select.update(cx, |s, _cx| s.select_demo_value = v);
                });
            });

            let state_for_combo = state.clone();
            s.combo_state.update(cx, |st, _cx| {
                st.set_on_change(move |value, _w, cx| {
                    let v = value.to_string();
                    state_for_combo.update(cx, |s, cx| {
                        s.combo_demo_value = v;
                        cx.notify();
                    });
                });
            });

            let state_for_editable_combo = state.clone();
            s.editable_combo_state.update(cx, |st, _cx| {
                st.set_on_change(move |value, _w, cx| {
                    let v = value.to_string();
                    state_for_editable_combo.update(cx, |s, cx| {
                        s.editable_combo_value = v;
                        cx.notify();
                    });
                });
            });

            let state_for_listbox = state.clone();
            s.listbox_state.update(cx, |st, _cx| {
                st.set_on_change(move |value, _w, cx| {
                    let v = value.to_string();
                    state_for_listbox.update(cx, |s, _cx| s.listbox_demo_value = v);
                });
            });

            let state_for_gridview = state.clone();
            s.gridview_state.update(cx, |st, _cx| {
                st.set_on_change(move |value, _w, cx| {
                    let v = value.to_string();
                    state_for_gridview.update(cx, |s, _cx| s.gridview_demo_value = v);
                });
            });

            // Popover menu: closes the popover and writes
            // `menu_demo_value` only.
            let state_for_menu = state.clone();
            let popover_state = s.popover_state.clone();
            s.menu_state.update(cx, |st, _cx| {
                st.set_on_select(move |id, _w, cx| {
                    let id_s = id.to_string();
                    state_for_menu.update(cx, |s, _cx| {
                        s.menu_demo_value = id_s;
                    });
                    popover_state.update(cx, |s, _cx| s.close());
                });
            });

            // Dropdown body menu: closes the dropdown and
            // writes `dropdown_demo_value` only. Kept
            // independent of `menu_state` so a click in the
            // dropdown's body does NOT also update the
            // popover's menu value.
            let state_for_dropdown_menu = state.clone();
            let dropdown_state = s.dropdown_state.clone();
            s.dropdown_menu_state.update(cx, |st, _cx| {
                st.set_on_select(move |id, _w, cx| {
                    let id_s = id.to_string();
                    state_for_dropdown_menu.update(cx, |s, _cx| {
                        s.dropdown_demo_value = id_s;
                    });
                    dropdown_state.update(cx, |s, _cx| s.close());
                });
            });
        });
    }
}
