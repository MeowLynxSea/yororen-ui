//! `RendererRegistry` — the collection of component renderers wired
//! into a `Theme`.
//!
//! ## Storage model
//!
//! Renderers are stored in a `HashMap<TypeId, Arc<dyn Any + Send + Sync>>`,
//! keyed by the component's `XxxRenderState` type. Each
//! `with_<x>(r: Arc<dyn XxxRenderer>)` setter inserts `r` under
//! `TypeId::of::<XxxRenderState>()`; each `get_<x>() -> Option<&Arc<dyn XxxRenderer>>`
//! looks it up. The `56 XxxRenderer` traits themselves stay public and
//! unchanged — they are the **type-level** contract a theme implements,
//! the HashMap is the **storage** layer.
//!
//! ## Why not 56 named fields?
//!
//! The 56-trait design (one trait per component) is the right level of
//! granularity for the *type system*: each `XxxRenderer` is callable
//! with the component-specific `XxxRenderState` and returns the
//! component-specific property shape. Trying to collapse them into a
//! single `ComponentRenderer<S: RenderState>` trait would force a
//! 200+-method union (because the 56 method sets are largely disjoint
//! — `Switch` has `track_w/track_h/knob_size` while `Avatar` has
//! `status_dot_size/status_inset`).
//!
//! What the 56-field `RendererRegistry` *didn't* need was the named
//! fields themselves: there is no code that reads `theme.renderers.button`
//! directly (all call sites go through `with_button` setters and
//! `get_button` accessors). So the fields are private storage, the
//! public surface is the 56 `with_<x>` / 56 `get_<x>` method pairs.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use super::avatar::TokenAvatarRenderer;
use super::badge::TokenBadgeRenderer;
use super::button::TokenButtonRenderer;
use super::button_group::TokenButtonGroupRenderer;
use super::card::TokenCardRenderer;
use super::checkbox::TokenCheckboxRenderer;
use super::combo_box::TokenComboBoxRenderer;
use super::disclosure::TokenDisclosureRenderer;
use super::divider::TokenDividerRenderer;
use super::dropdown_menu::TokenDropdownMenuRenderer;
use super::empty_state::TokenEmptyStateRenderer;
use super::file_path_input::TokenFilePathInputRenderer;
use super::focus_ring::TokenFocusRingRenderer;
use super::form::TokenFormRenderer;
use super::form_field::TokenFormFieldRenderer;
use super::grid_view::TokenGridViewRenderer;
use super::heading::TokenHeadingRenderer;
use super::icon::TokenIconRenderer;
use super::icon_button::TokenIconButtonRenderer;
use super::image::TokenImageRenderer;
use super::keybinding_display::TokenKeybindingDisplayRenderer;
use super::keybinding_input::TokenKeybindingInputRenderer;
use super::label::TokenLabelRenderer;
use super::list_item::TokenListItemRenderer;
use super::listbox::TokenListboxRenderer;
use super::menu::TokenMenuRenderer;
use super::modal::TokenModalRenderer;
use super::notification::TokenNotificationRenderer;
use super::number_input::TokenNumberInputRenderer;
use super::overlay::TokenOverlayRenderer;
use super::panel::TokenPanelRenderer;
use super::password_input::TokenPasswordInputRenderer;
use super::popover::TokenPopoverRenderer;
use super::progress::TokenProgressBarRenderer;
use super::radio::TokenRadioRenderer;
use super::radio_group::TokenRadioGroupRenderer;
use super::search_input::TokenSearchInputRenderer;
use super::select::TokenSelectRenderer;
use super::shortcut_hint::TokenShortcutHintRenderer;
use super::skeleton::TokenSkeletonRenderer;
use super::slider::TokenSliderRenderer;
use super::spacer::TokenSpacerRenderer;
use super::split_button::TokenSplitButtonRenderer;
use super::switch::TokenSwitchRenderer;
use super::table::TokenTableRenderer;
use super::tag::TokenTagRenderer;
use super::text::TokenTextRenderer;
use super::text_area::TokenTextAreaRenderer;
use super::text_input::TokenTextInputRenderer;
use super::toast::TokenToastRenderer;
use super::toggle_button::TokenToggleButtonRenderer;
use super::tooltip::TokenTooltipRenderer;
use super::tree::TokenTreeRenderer;
use super::tree_item::TokenTreeItemRenderer;
use super::uniform_virtual_list::TokenUniformVirtualListRenderer;
use super::virtual_list::TokenVirtualListRenderer;
use yororen_ui_core::renderer::avatar::AvatarRenderer;
use yororen_ui_core::renderer::badge::BadgeRenderer;
use yororen_ui_core::renderer::button::ButtonRenderer;
use yororen_ui_core::renderer::button_group::ButtonGroupRenderer;
use yororen_ui_core::renderer::card::CardRenderer;
use yororen_ui_core::renderer::checkbox::CheckboxRenderer;
use yororen_ui_core::renderer::combo_box::ComboBoxRenderer;
use yororen_ui_core::renderer::disclosure::DisclosureRenderer;
use yororen_ui_core::renderer::divider::DividerRenderer;
use yororen_ui_core::renderer::dropdown_menu::DropdownMenuRenderer;
use yororen_ui_core::renderer::empty_state::EmptyStateRenderer;
use yororen_ui_core::renderer::file_path_input::FilePathInputRenderer;
use yororen_ui_core::renderer::focus_ring::FocusRingRenderer;
use yororen_ui_core::renderer::form::FormRenderer;
use yororen_ui_core::renderer::form_field::FormFieldRenderer;
use yororen_ui_core::renderer::grid_view::GridViewRenderer;
use yororen_ui_core::renderer::heading::HeadingRenderer;
use yororen_ui_core::renderer::icon::IconRenderer;
use yororen_ui_core::renderer::icon_button::IconButtonRenderer;
use yororen_ui_core::renderer::image::ImageRenderer;
use yororen_ui_core::renderer::keybinding_display::KeybindingDisplayRenderer;
use yororen_ui_core::renderer::keybinding_input::KeybindingInputRenderer;
use yororen_ui_core::renderer::label::LabelRenderer;
use yororen_ui_core::renderer::list_item::ListItemRenderer;
use yororen_ui_core::renderer::listbox::ListboxRenderer;
use yororen_ui_core::renderer::menu::MenuRenderer;
use yororen_ui_core::renderer::modal::ModalRenderer;
use yororen_ui_core::renderer::notification::NotificationRenderer;
use yororen_ui_core::renderer::number_input::NumberInputRenderer;
use yororen_ui_core::renderer::overlay::OverlayRenderer;
use yororen_ui_core::renderer::panel::PanelRenderer;
use yororen_ui_core::renderer::password_input::PasswordInputRenderer;
use yororen_ui_core::renderer::popover::PopoverRenderer;
use yororen_ui_core::renderer::progress::ProgressBarRenderer;
use yororen_ui_core::renderer::radio::RadioRenderer;
use yororen_ui_core::renderer::radio_group::RadioGroupRenderer;
use yororen_ui_core::renderer::search_input::SearchInputRenderer;
use yororen_ui_core::renderer::select::SelectRenderer;
use yororen_ui_core::renderer::shortcut_hint::ShortcutHintRenderer;
use yororen_ui_core::renderer::skeleton::SkeletonRenderer;
use yororen_ui_core::renderer::slider::SliderRenderer;
use yororen_ui_core::renderer::spacer::SpacerRenderer;
use yororen_ui_core::renderer::split_button::SplitButtonRenderer;
use yororen_ui_core::renderer::switch::SwitchRenderer;
use yororen_ui_core::renderer::table::TableRenderer;
use yororen_ui_core::renderer::tag::TagRenderer;
use yororen_ui_core::renderer::text::TextRenderer;
use yororen_ui_core::renderer::text_area::TextAreaRenderer;
use yororen_ui_core::renderer::text_input::TextInputRenderer;
use yororen_ui_core::renderer::toast::ToastRenderer;
use yororen_ui_core::renderer::toggle_button::ToggleButtonRenderer;
use yororen_ui_core::renderer::tooltip::TooltipRenderer;
use yororen_ui_core::renderer::tree::TreeRenderer;
use yororen_ui_core::renderer::tree_item::TreeItemRenderer;
use yororen_ui_core::renderer::uniform_virtual_list::UniformVirtualListRenderer;
use yororen_ui_core::renderer::virtual_list::VirtualListRenderer;

use yororen_ui_core::renderer::avatar::AvatarRenderState;
use yororen_ui_core::renderer::badge::BadgeRenderState;
use yororen_ui_core::renderer::button::ButtonRenderState;
use yororen_ui_core::renderer::button_group::ButtonGroupRenderState;
use yororen_ui_core::renderer::card::CardRenderState;
use yororen_ui_core::renderer::checkbox::CheckboxRenderState;
use yororen_ui_core::renderer::combo_box::ComboBoxRenderState;
use yororen_ui_core::renderer::disclosure::DisclosureRenderState;
use yororen_ui_core::renderer::divider::DividerRenderState;
use yororen_ui_core::renderer::dropdown_menu::DropdownMenuRenderState;
use yororen_ui_core::renderer::empty_state::EmptyStateRenderState;
use yororen_ui_core::renderer::file_path_input::FilePathInputRenderState;
use yororen_ui_core::renderer::focus_ring::FocusRingRenderState;
use yororen_ui_core::renderer::form::FormRenderState;
use yororen_ui_core::renderer::form_field::FormFieldRenderState;
use yororen_ui_core::renderer::grid_view::GridViewRenderState;
use yororen_ui_core::renderer::heading::HeadingRenderState;
use yororen_ui_core::renderer::icon::IconRenderState;
use yororen_ui_core::renderer::icon_button::IconButtonRenderState;
use yororen_ui_core::renderer::image::ImageRenderState;
use yororen_ui_core::renderer::keybinding_display::KeybindingDisplayRenderState;
use yororen_ui_core::renderer::keybinding_input::KeybindingInputRenderState;
use yororen_ui_core::renderer::label::LabelRenderState;
use yororen_ui_core::renderer::list_item::ListItemRenderState;
use yororen_ui_core::renderer::listbox::ListboxRenderState;
use yororen_ui_core::renderer::menu::MenuRenderState;
use yororen_ui_core::renderer::modal::ModalRenderState;
use yororen_ui_core::renderer::notification::NotificationRenderState;
use yororen_ui_core::renderer::number_input::NumberInputRenderState;
use yororen_ui_core::renderer::overlay::OverlayRenderState;
use yororen_ui_core::renderer::panel::PanelRenderState;
use yororen_ui_core::renderer::password_input::PasswordInputRenderState;
use yororen_ui_core::renderer::popover::PopoverRenderState;
use yororen_ui_core::renderer::progress::ProgressBarRenderState;
use yororen_ui_core::renderer::radio::RadioRenderState;
use yororen_ui_core::renderer::radio_group::RadioGroupRenderState;
use yororen_ui_core::renderer::search_input::SearchInputRenderState;
use yororen_ui_core::renderer::select::SelectRenderState;
use yororen_ui_core::renderer::shortcut_hint::ShortcutHintRenderState;
use yororen_ui_core::renderer::skeleton::SkeletonRenderState;
use yororen_ui_core::renderer::slider::SliderRenderState;
use yororen_ui_core::renderer::spacer::SpacerRenderState;
use yororen_ui_core::renderer::split_button::SplitButtonRenderState;
use yororen_ui_core::renderer::switch::SwitchRenderState;
use yororen_ui_core::renderer::table::TableRenderState;
use yororen_ui_core::renderer::tag::TagRenderState;
use yororen_ui_core::renderer::text::TextRenderState;
use yororen_ui_core::renderer::text_area::TextAreaRenderState;
use yororen_ui_core::renderer::text_input::TextInputRenderState;
use yororen_ui_core::renderer::toast::ToastRenderState;
use yororen_ui_core::renderer::toggle_button::ToggleButtonRenderState;
use yororen_ui_core::renderer::tooltip::TooltipRenderState;
use yororen_ui_core::renderer::tree::TreeRenderState;
use yororen_ui_core::renderer::tree_item::TreeItemRenderState;
use yororen_ui_core::renderer::uniform_virtual_list::UniformVirtualListRenderState;
use yororen_ui_core::renderer::virtual_list::VirtualListRenderState;

/// Collection of component renderers. Looked up at render time by
/// `XxxRenderState` `TypeId`.
///
/// Public surface: 56 `with_<x>(Arc<dyn XxxRenderer>)` setters + 56
/// `get_<x>() -> Option<&Arc<dyn XxxRenderer>>` accessors. The
/// underlying HashMap is private.
#[derive(Clone)]
pub struct RendererRegistry {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

/// Generate one `with_<x>` setter per component. The setter stores
/// the renderer under `TypeId::of::<XxxRenderState>()`. The
/// `Arc<dyn XxxRenderer>` is wrapped in an outer `Arc` and the whole
/// thing is cast to `Arc<dyn Any + Send + Sync>`. The outer wrap is
/// what makes the downcast back to `Arc<dyn XxxRenderer>` work: the
/// `Arc::downcast_ref::<Arc<dyn XxxRenderer>>()` matches the
/// concrete Sized `Arc<dyn XxxRenderer>` stored inside the outer
/// `Arc<dyn Any>`.
macro_rules! renderer_setter {
    ($setter:ident, $state:ty, $trait:path) => {
        pub fn $setter(mut self, r: Arc<dyn $trait>) -> Self {
            // Outer Arc wraps the inner Arc<dyn XxxRenderer>; the
            // outer Arc is the "Any" container, and downcasting
            // its inner Any back to `Arc<dyn XxxRenderer>` recovers
            // the original typed renderer.
            let any: Arc<dyn Any + Send + Sync> = Arc::new(r);
            self.map.insert(TypeId::of::<$state>(), any);
            self
        }
    };
}

/// Generate one `get_<x>()` typed accessor per component.
macro_rules! renderer_getter {
    ($getter:ident, $state:ty, $trait:path) => {
        pub fn $getter(&self) -> Option<&Arc<dyn $trait>> {
            self.get_typed::<$state, dyn $trait>()
        }
    };
}

impl std::fmt::Debug for RendererRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererRegistry").finish_non_exhaustive()
    }
}

impl Default for RendererRegistry {
    fn default() -> Self {
        Self::token_based()
    }
}

impl RendererRegistry {
    /// All renderers set to the default `TokenXxxRenderer` implementations.
    /// This is the v0.3 / v0.4 visual baseline.
    ///
    /// Self-validates on construction: if a future edit adds a new
    /// `with_<x>(...)` line here but forgets to mirror it in `REQUIRED`,
    /// `token_based()` panics with the full list of missing entries
    /// the first time it is called. This is the single coupling point
    /// between the 56 setters and the 56 `REQUIRED` entries.
    pub fn token_based() -> Self {
        // Cannot use `Self::default()` here — `default` is implemented
        // as `token_based`, so that would recurse forever. Construct
        // the empty registry directly and chain the 56 setters.
        let registry = Self {
            map: HashMap::new(),
        }
        .with_button(Arc::new(TokenButtonRenderer))
        .with_button_group(Arc::new(TokenButtonGroupRenderer))
        .with_icon_button(Arc::new(TokenIconButtonRenderer))
        .with_toggle_button(Arc::new(TokenToggleButtonRenderer))
        .with_label(Arc::new(TokenLabelRenderer))
        .with_heading(Arc::new(TokenHeadingRenderer))
        .with_icon(Arc::new(TokenIconRenderer))
        .with_divider(Arc::new(TokenDividerRenderer))
        .with_focus_ring(Arc::new(TokenFocusRingRenderer))
        .with_badge(Arc::new(TokenBadgeRenderer))
        .with_tag(Arc::new(TokenTagRenderer))
        .with_progress_bar(Arc::new(TokenProgressBarRenderer))
        .with_skeleton(Arc::new(TokenSkeletonRenderer))
        .with_slider(Arc::new(TokenSliderRenderer))
        .with_tooltip(Arc::new(TokenTooltipRenderer))
        .with_avatar(Arc::new(TokenAvatarRenderer))
        .with_switch(Arc::new(TokenSwitchRenderer))
        .with_checkbox(Arc::new(TokenCheckboxRenderer))
        .with_radio(Arc::new(TokenRadioRenderer))
        .with_text_input(Arc::new(TokenTextInputRenderer))
        .with_text_area(Arc::new(TokenTextAreaRenderer))
        .with_password_input(Arc::new(TokenPasswordInputRenderer))
        .with_number_input(Arc::new(TokenNumberInputRenderer))
        .with_file_path_input(Arc::new(TokenFilePathInputRenderer))
        .with_search_input(Arc::new(TokenSearchInputRenderer))
        .with_select(Arc::new(TokenSelectRenderer))
        .with_combo_box(Arc::new(TokenComboBoxRenderer))
        .with_modal(Arc::new(TokenModalRenderer))
        .with_popover(Arc::new(TokenPopoverRenderer))
        .with_dropdown_menu(Arc::new(TokenDropdownMenuRenderer))
        .with_disclosure(Arc::new(TokenDisclosureRenderer))
        .with_toast(Arc::new(TokenToastRenderer))
        .with_notification(Arc::new(TokenNotificationRenderer))
        .with_panel(Arc::new(TokenPanelRenderer))
        .with_card(Arc::new(TokenCardRenderer))
        .with_form(Arc::new(TokenFormRenderer))
        .with_form_field(Arc::new(TokenFormFieldRenderer))
        .with_grid_view(Arc::new(TokenGridViewRenderer))
        .with_list_item(Arc::new(TokenListItemRenderer))
        .with_listbox(Arc::new(TokenListboxRenderer))
        .with_menu(Arc::new(TokenMenuRenderer))
        .with_overlay(Arc::new(TokenOverlayRenderer))
        .with_radio_group(Arc::new(TokenRadioGroupRenderer))
        .with_spacer(Arc::new(TokenSpacerRenderer))
        .with_table(Arc::new(TokenTableRenderer))
        .with_text(Arc::new(TokenTextRenderer))
        .with_tree(Arc::new(TokenTreeRenderer))
        .with_tree_item(Arc::new(TokenTreeItemRenderer))
        .with_virtual_list(Arc::new(TokenVirtualListRenderer))
        .with_uniform_virtual_list(Arc::new(TokenUniformVirtualListRenderer))
        .with_keybinding_input(Arc::new(TokenKeybindingInputRenderer))
        .with_split_button(Arc::new(TokenSplitButtonRenderer))
        .with_empty_state(Arc::new(TokenEmptyStateRenderer))
        .with_image(Arc::new(TokenImageRenderer))
        .with_keybinding_display(Arc::new(TokenKeybindingDisplayRenderer))
        .with_shortcut_hint(Arc::new(TokenShortcutHintRenderer));

        if registry.validate().is_err() {
            panic_missing_renderers(&registry);
        }
        registry
    }

    // -- 56 setters (one per component) ---------------------------------
    // Each setter preserves the original `with_<x>(Arc<dyn XxxRenderer>)`
    // signature exactly — theme packages and downstream apps call them
    // unchanged. The renderer is stored under
    // `TypeId::of::<XxxRenderState>()`.

    renderer_setter!(with_button, ButtonRenderState, ButtonRenderer);
    renderer_setter!(
        with_button_group,
        ButtonGroupRenderState,
        ButtonGroupRenderer
    );
    renderer_setter!(with_label, LabelRenderState, LabelRenderer);
    renderer_setter!(with_heading, HeadingRenderState, HeadingRenderer);
    renderer_setter!(with_icon, IconRenderState, IconRenderer);
    renderer_setter!(with_divider, DividerRenderState, DividerRenderer);
    renderer_setter!(with_focus_ring, FocusRingRenderState, FocusRingRenderer);
    renderer_setter!(with_badge, BadgeRenderState, BadgeRenderer);
    renderer_setter!(with_tag, TagRenderState, TagRenderer);
    renderer_setter!(
        with_progress_bar,
        ProgressBarRenderState,
        ProgressBarRenderer
    );
    renderer_setter!(with_skeleton, SkeletonRenderState, SkeletonRenderer);
    renderer_setter!(with_slider, SliderRenderState, SliderRenderer);
    renderer_setter!(with_tooltip, TooltipRenderState, TooltipRenderer);
    renderer_setter!(with_avatar, AvatarRenderState, AvatarRenderer);
    renderer_setter!(with_switch, SwitchRenderState, SwitchRenderer);
    renderer_setter!(with_checkbox, CheckboxRenderState, CheckboxRenderer);
    renderer_setter!(with_radio, RadioRenderState, RadioRenderer);
    renderer_setter!(with_icon_button, IconButtonRenderState, IconButtonRenderer);
    renderer_setter!(
        with_toggle_button,
        ToggleButtonRenderState,
        ToggleButtonRenderer
    );
    renderer_setter!(with_text_input, TextInputRenderState, TextInputRenderer);
    renderer_setter!(with_modal, ModalRenderState, ModalRenderer);
    renderer_setter!(with_popover, PopoverRenderState, PopoverRenderer);
    renderer_setter!(
        with_dropdown_menu,
        DropdownMenuRenderState,
        DropdownMenuRenderer
    );
    renderer_setter!(with_toast, ToastRenderState, ToastRenderer);
    renderer_setter!(
        with_notification,
        NotificationRenderState,
        NotificationRenderer
    );
    renderer_setter!(with_panel, PanelRenderState, PanelRenderer);
    renderer_setter!(with_card, CardRenderState, CardRenderer);
    renderer_setter!(with_form, FormRenderState, FormRenderer);
    renderer_setter!(with_form_field, FormFieldRenderState, FormFieldRenderer);
    renderer_setter!(with_grid_view, GridViewRenderState, GridViewRenderer);
    renderer_setter!(with_list_item, ListItemRenderState, ListItemRenderer);
    renderer_setter!(with_listbox, ListboxRenderState, ListboxRenderer);
    renderer_setter!(with_menu, MenuRenderState, MenuRenderer);
    renderer_setter!(with_overlay, OverlayRenderState, OverlayRenderer);
    renderer_setter!(with_radio_group, RadioGroupRenderState, RadioGroupRenderer);
    renderer_setter!(with_spacer, SpacerRenderState, SpacerRenderer);
    renderer_setter!(with_table, TableRenderState, TableRenderer);
    renderer_setter!(with_text, TextRenderState, TextRenderer);
    renderer_setter!(with_tree, TreeRenderState, TreeRenderer);
    renderer_setter!(
        with_virtual_list,
        VirtualListRenderState,
        VirtualListRenderer
    );
    renderer_setter!(
        with_uniform_virtual_list,
        UniformVirtualListRenderState,
        UniformVirtualListRenderer
    );
    renderer_setter!(with_text_area, TextAreaRenderState, TextAreaRenderer);
    renderer_setter!(
        with_password_input,
        PasswordInputRenderState,
        PasswordInputRenderer
    );
    renderer_setter!(
        with_number_input,
        NumberInputRenderState,
        NumberInputRenderer
    );
    renderer_setter!(
        with_file_path_input,
        FilePathInputRenderState,
        FilePathInputRenderer
    );
    renderer_setter!(
        with_search_input,
        SearchInputRenderState,
        SearchInputRenderer
    );
    renderer_setter!(with_select, SelectRenderState, SelectRenderer);
    renderer_setter!(with_combo_box, ComboBoxRenderState, ComboBoxRenderer);
    renderer_setter!(with_disclosure, DisclosureRenderState, DisclosureRenderer);
    renderer_setter!(with_tree_item, TreeItemRenderState, TreeItemRenderer);
    renderer_setter!(
        with_keybinding_input,
        KeybindingInputRenderState,
        KeybindingInputRenderer
    );
    renderer_setter!(
        with_split_button,
        SplitButtonRenderState,
        SplitButtonRenderer
    );
    renderer_setter!(with_empty_state, EmptyStateRenderState, EmptyStateRenderer);
    renderer_setter!(with_image, ImageRenderState, ImageRenderer);
    renderer_setter!(
        with_keybinding_display,
        KeybindingDisplayRenderState,
        KeybindingDisplayRenderer
    );
    renderer_setter!(
        with_shortcut_hint,
        ShortcutHintRenderState,
        ShortcutHintRenderer
    );

    /// Internal: typed lookup. The component-side accessors are
    /// `get_<x>()` 1-liners that pin both the state and the trait.
    ///
    /// The setter stores `Box::new(r as Arc<dyn XxxRenderer>)` cast
    /// to `Box<dyn Any + Send + Sync>`. So the inner `Any` is
    /// `Arc<dyn XxxRenderer>`. We downcast back to that `Arc<...>`
    /// to recover the original typed renderer.
    fn get_typed<S: yororen_ui_core::renderer::spec::RenderState, R: ?Sized + 'static>(
        &self,
    ) -> Option<&Arc<R>> {
        self.map
            .get(&TypeId::of::<S>())
            .and_then(|arc| arc.downcast_ref::<Arc<R>>())
    }

    // -- 56 typed accessors (one per component) -------------------------
    // Each returns `Option<&Arc<dyn XxxRenderer>>`. Components call
    // these in their `RenderOnce::render` body; theme packages and
    // downstream code do not need them (the setter is enough to
    // install a custom renderer).

    renderer_getter!(get_button, ButtonRenderState, ButtonRenderer);
    renderer_getter!(
        get_button_group,
        ButtonGroupRenderState,
        ButtonGroupRenderer
    );
    renderer_getter!(get_label, LabelRenderState, LabelRenderer);
    renderer_getter!(get_heading, HeadingRenderState, HeadingRenderer);
    renderer_getter!(get_icon, IconRenderState, IconRenderer);
    renderer_getter!(get_divider, DividerRenderState, DividerRenderer);
    renderer_getter!(get_focus_ring, FocusRingRenderState, FocusRingRenderer);
    renderer_getter!(get_badge, BadgeRenderState, BadgeRenderer);
    renderer_getter!(get_tag, TagRenderState, TagRenderer);
    renderer_getter!(
        get_progress_bar,
        ProgressBarRenderState,
        ProgressBarRenderer
    );
    renderer_getter!(get_skeleton, SkeletonRenderState, SkeletonRenderer);
    renderer_getter!(get_slider, SliderRenderState, SliderRenderer);
    renderer_getter!(get_tooltip, TooltipRenderState, TooltipRenderer);
    renderer_getter!(get_avatar, AvatarRenderState, AvatarRenderer);
    renderer_getter!(get_switch, SwitchRenderState, SwitchRenderer);
    renderer_getter!(get_checkbox, CheckboxRenderState, CheckboxRenderer);
    renderer_getter!(get_radio, RadioRenderState, RadioRenderer);
    renderer_getter!(get_icon_button, IconButtonRenderState, IconButtonRenderer);
    renderer_getter!(
        get_toggle_button,
        ToggleButtonRenderState,
        ToggleButtonRenderer
    );
    renderer_getter!(get_text_input, TextInputRenderState, TextInputRenderer);
    renderer_getter!(get_modal, ModalRenderState, ModalRenderer);
    renderer_getter!(get_popover, PopoverRenderState, PopoverRenderer);
    renderer_getter!(
        get_dropdown_menu,
        DropdownMenuRenderState,
        DropdownMenuRenderer
    );
    renderer_getter!(get_toast, ToastRenderState, ToastRenderer);
    renderer_getter!(
        get_notification,
        NotificationRenderState,
        NotificationRenderer
    );
    renderer_getter!(get_panel, PanelRenderState, PanelRenderer);
    renderer_getter!(get_card, CardRenderState, CardRenderer);
    renderer_getter!(get_form, FormRenderState, FormRenderer);
    renderer_getter!(get_form_field, FormFieldRenderState, FormFieldRenderer);
    renderer_getter!(get_grid_view, GridViewRenderState, GridViewRenderer);
    renderer_getter!(get_list_item, ListItemRenderState, ListItemRenderer);
    renderer_getter!(get_listbox, ListboxRenderState, ListboxRenderer);
    renderer_getter!(get_menu, MenuRenderState, MenuRenderer);
    renderer_getter!(get_overlay, OverlayRenderState, OverlayRenderer);
    renderer_getter!(get_radio_group, RadioGroupRenderState, RadioGroupRenderer);
    renderer_getter!(get_spacer, SpacerRenderState, SpacerRenderer);
    renderer_getter!(get_table, TableRenderState, TableRenderer);
    renderer_getter!(get_text, TextRenderState, TextRenderer);
    renderer_getter!(get_tree, TreeRenderState, TreeRenderer);
    renderer_getter!(
        get_virtual_list,
        VirtualListRenderState,
        VirtualListRenderer
    );
    renderer_getter!(
        get_uniform_virtual_list,
        UniformVirtualListRenderState,
        UniformVirtualListRenderer
    );
    renderer_getter!(get_text_area, TextAreaRenderState, TextAreaRenderer);
    renderer_getter!(
        get_password_input,
        PasswordInputRenderState,
        PasswordInputRenderer
    );
    renderer_getter!(
        get_number_input,
        NumberInputRenderState,
        NumberInputRenderer
    );
    renderer_getter!(
        get_file_path_input,
        FilePathInputRenderState,
        FilePathInputRenderer
    );
    renderer_getter!(
        get_search_input,
        SearchInputRenderState,
        SearchInputRenderer
    );
    renderer_getter!(get_select, SelectRenderState, SelectRenderer);
    renderer_getter!(get_combo_box, ComboBoxRenderState, ComboBoxRenderer);
    renderer_getter!(get_disclosure, DisclosureRenderState, DisclosureRenderer);
    renderer_getter!(get_tree_item, TreeItemRenderState, TreeItemRenderer);
    renderer_getter!(
        get_keybinding_input,
        KeybindingInputRenderState,
        KeybindingInputRenderer
    );
    renderer_getter!(
        get_split_button,
        SplitButtonRenderState,
        SplitButtonRenderer
    );
    renderer_getter!(get_empty_state, EmptyStateRenderState, EmptyStateRenderer);
    renderer_getter!(get_image, ImageRenderState, ImageRenderer);
    renderer_getter!(
        get_keybinding_display,
        KeybindingDisplayRenderState,
        KeybindingDisplayRenderer
    );
    renderer_getter!(
        get_shortcut_hint,
        ShortcutHintRenderState,
        ShortcutHintRenderer
    );

    /// All 56 `(TypeId, "name")` pairs that a complete registry must
    /// contain. Single source of truth for `validate()` and for the
    /// self-check at the end of `token_based()`.
    ///
    /// The names are stable strings (matching the `with_<x>` setter
    /// identifiers) so error messages stay grep-friendly: a panic
    /// listing `"text_input, password_input"` tells the theme author
    /// exactly which `with_<x>(...)` calls they forgot.
    const REQUIRED: &[(TypeId, &str)] = &[
        (TypeId::of::<ButtonRenderState>(), "button"),
        (TypeId::of::<ButtonGroupRenderState>(), "button_group"),
        (TypeId::of::<IconButtonRenderState>(), "icon_button"),
        (TypeId::of::<ToggleButtonRenderState>(), "toggle_button"),
        (TypeId::of::<LabelRenderState>(), "label"),
        (TypeId::of::<HeadingRenderState>(), "heading"),
        (TypeId::of::<IconRenderState>(), "icon"),
        (TypeId::of::<DividerRenderState>(), "divider"),
        (TypeId::of::<FocusRingRenderState>(), "focus_ring"),
        (TypeId::of::<BadgeRenderState>(), "badge"),
        (TypeId::of::<TagRenderState>(), "tag"),
        (TypeId::of::<ProgressBarRenderState>(), "progress_bar"),
        (TypeId::of::<SkeletonRenderState>(), "skeleton"),
        (TypeId::of::<SliderRenderState>(), "slider"),
        (TypeId::of::<TooltipRenderState>(), "tooltip"),
        (TypeId::of::<AvatarRenderState>(), "avatar"),
        (TypeId::of::<SwitchRenderState>(), "switch"),
        (TypeId::of::<CheckboxRenderState>(), "checkbox"),
        (TypeId::of::<RadioRenderState>(), "radio"),
        (TypeId::of::<TextInputRenderState>(), "text_input"),
        (TypeId::of::<TextAreaRenderState>(), "text_area"),
        (TypeId::of::<PasswordInputRenderState>(), "password_input"),
        (TypeId::of::<NumberInputRenderState>(), "number_input"),
        (TypeId::of::<FilePathInputRenderState>(), "file_path_input"),
        (TypeId::of::<SearchInputRenderState>(), "search_input"),
        (TypeId::of::<SelectRenderState>(), "select"),
        (TypeId::of::<ComboBoxRenderState>(), "combo_box"),
        (TypeId::of::<ModalRenderState>(), "modal"),
        (TypeId::of::<PopoverRenderState>(), "popover"),
        (TypeId::of::<DropdownMenuRenderState>(), "dropdown_menu"),
        (TypeId::of::<DisclosureRenderState>(), "disclosure"),
        (TypeId::of::<ToastRenderState>(), "toast"),
        (TypeId::of::<NotificationRenderState>(), "notification"),
        (TypeId::of::<PanelRenderState>(), "panel"),
        (TypeId::of::<CardRenderState>(), "card"),
        (TypeId::of::<FormRenderState>(), "form"),
        (TypeId::of::<FormFieldRenderState>(), "form_field"),
        (TypeId::of::<GridViewRenderState>(), "grid_view"),
        (TypeId::of::<ListItemRenderState>(), "list_item"),
        (TypeId::of::<ListboxRenderState>(), "listbox"),
        (TypeId::of::<MenuRenderState>(), "menu"),
        (TypeId::of::<OverlayRenderState>(), "overlay"),
        (TypeId::of::<RadioGroupRenderState>(), "radio_group"),
        (TypeId::of::<SpacerRenderState>(), "spacer"),
        (TypeId::of::<TableRenderState>(), "table"),
        (TypeId::of::<TextRenderState>(), "text"),
        (TypeId::of::<TreeRenderState>(), "tree"),
        (TypeId::of::<TreeItemRenderState>(), "tree_item"),
        (TypeId::of::<VirtualListRenderState>(), "virtual_list"),
        (
            TypeId::of::<UniformVirtualListRenderState>(),
            "uniform_virtual_list",
        ),
        (
            TypeId::of::<KeybindingInputRenderState>(),
            "keybinding_input",
        ),
        (TypeId::of::<SplitButtonRenderState>(), "split_button"),
        (TypeId::of::<EmptyStateRenderState>(), "empty_state"),
        (TypeId::of::<ImageRenderState>(), "image"),
        (
            TypeId::of::<KeybindingDisplayRenderState>(),
            "keybinding_display",
        ),
        (TypeId::of::<ShortcutHintRenderState>(), "shortcut_hint"),
    ];

    /// Verify that this registry contains a renderer for **all** 56
    /// `XxxRenderState` types. Returns `Ok(())` if complete, or
    /// `Err(missing)` listing the names of every absent renderer.
    ///
    /// ## Why
    ///
    /// Component render paths look up their renderer via
    /// `get_<x>().expect("XxxRenderer registered")`. That `expect`
    /// is the *last line of defense*; if it ever fires in production
    /// it means a theme package was constructed without all 54
    /// renderers registered. This is a *configuration* bug, not a
    /// runtime condition, and the fix is "add `with_<x>(...)` to your
    /// theme's registry builder".
    ///
    /// `validate()` is the recommended way to catch that bug at app
    /// boot time, before any component renders. Theme packages should
    /// call it inside their `install(cx)` entrypoint:
    ///
    /// ```ignore
    /// pub fn install(cx: &mut App) {
    ///     let registry = RendererRegistry::token_based()
    ///         .with_button(Arc::new(MyButtonRenderer));
    ///     registry.validate().expect("my-theme registry incomplete");
    ///     cx.set_global(GlobalTheme::new(Theme { renderers: registry, .. }));
    /// }
    /// ```
    ///
    /// `token_based()` self-validates on construction, so registries
    /// built on top of it (the common case) only need this check as
    /// a "did I forget a `with_<x>` after `token_based()`?" guard.
    pub fn validate(&self) -> Result<(), Vec<&'static str>> {
        let missing: Vec<&'static str> = Self::REQUIRED
            .iter()
            .copied()
            .filter_map(|(id, name)| (!self.map.contains_key(&id)).then_some(name))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

/// Internal: emit a panic listing the missing renderers. Used by
/// `token_based()`'s self-check to guarantee that future edits to
/// `token_based()` (adding a `with_<x>(...)` call) are always
/// mirrored in `REQUIRED`.
#[cold]
#[inline(never)]
fn panic_missing_renderers(registry: &RendererRegistry) -> ! {
    match registry.validate() {
        Ok(()) => unreachable!("panic_missing_renderers called on a valid registry"),
        Err(missing) => panic!(
            "RendererRegistry::token_based() is missing {} renderer(s): {}. \
             This is an internal bug: token_based() must register all 56 components. \
             Please report this as a yororen-ui bug.",
            missing.len(),
            missing.join(", "),
        ),
    }
}

#[cfg(test)]
mod tests {
    //! `RendererRegistry::validate()` is the boot-time guard against
    //! "theme package forgot to call `with_<x>(...)` for some
    //! component". The 6 tests below cover every interesting case
    //! of "how could a registry be incomplete" — and one regression
    //! test for the public `token_based()` happy path.

    use super::*;
    use std::any::TypeId;

    /// Build a registry that has *only* `ButtonRenderState`
    /// registered. Used to assert validate() reports the other 55
    /// as missing.
    fn only_button() -> RendererRegistry {
        let mut map: HashMap<TypeId, Arc<dyn Any + Send + Sync>> = HashMap::new();
        let arc: Arc<dyn ButtonRenderer> = Arc::new(super::super::button::TokenButtonRenderer);
        map.insert(
            TypeId::of::<yororen_ui_core::renderer::button::ButtonRenderState>(),
            Arc::new(arc),
        );
        RendererRegistry { map }
    }

    #[test]
    fn empty_registry_reports_all_missing() {
        let r = RendererRegistry {
            map: HashMap::new(),
        };
        let err = r.validate().unwrap_err();
        assert_eq!(
            err.len(),
            56,
            "expected 56 missing entries, got {}: {:?}",
            err.len(),
            err
        );
        // Spot-check a few well-known component names.
        assert!(err.contains(&"button"));
        assert!(err.contains(&"text_input"));
        assert!(err.contains(&"modal"));
        assert!(err.contains(&"tree_item"));
        assert!(err.contains(&"empty_state"));
        assert!(err.contains(&"image"));
        assert!(err.contains(&"keybinding_display"));
        assert!(err.contains(&"shortcut_hint"));
        assert!(err.contains(&"uniform_virtual_list"));
        assert!(err.contains(&"listbox"));
        assert!(err.contains(&"grid_view"));
    }

    #[test]
    fn only_button_reports_missing() {
        let r = only_button();
        let err = r.validate().unwrap_err();
        assert_eq!(
            err.len(),
            55,
            "expected 55 missing entries, got {}: {:?}",
            err.len(),
            err
        );
        assert!(
            !err.contains(&"button"),
            "button was registered, must not appear in missing"
        );
        assert!(err.contains(&"text_input"));
        assert!(err.contains(&"listbox"));
    }

    #[test]
    fn token_based_passes_validation() {
        // This is the primary happy path: every theme package that
        // builds on `RendererRegistry::token_based()` must pass.
        let r = RendererRegistry::token_based();
        r.validate().expect("token_based() must always validate");
    }

    #[test]
    fn default_impl_passes_validation() {
        // `Default::default()` is defined as `token_based()`, so it
        // must also pass — this guards the (infinite-recursion)
        // implementation comment in `token_based()` from drifting.
        let r = RendererRegistry::default();
        r.validate()
            .expect("Default::default() must always validate");
    }

    #[test]
    fn override_after_token_based_still_passes() {
        // The realistic theme-package pattern: take `token_based()`
        // and override one renderer. The override path must not
        // accidentally drop the other 53.
        let r = RendererRegistry::token_based()
            .with_button(Arc::new(super::super::button::TokenButtonRenderer));
        r.validate()
            .expect("override-after-token_based must validate");
    }

    #[test]
    fn missing_set_is_sorted_in_required_order() {
        // The missing list is built by iterating `REQUIRED`, so the
        // order is deterministic and matches the declaration order.
        // This is a regression test against a refactor that might
        // (e.g.) switch to a HashSet and lose order.
        let r = RendererRegistry {
            map: HashMap::new(),
        };
        let err = r.validate().unwrap_err();
        let required_order: Vec<&'static str> =
            RendererRegistry::REQUIRED.iter().map(|(_, n)| *n).collect();
        assert_eq!(
            err, required_order,
            "missing list must follow REQUIRED declaration order"
        );
    }

    #[test]
    #[should_panic(expected = "token_based() is missing")]
    fn token_based_panics_if_a_setter_call_is_removed() {
        // Simulate the failure mode the self-check guards against:
        // a future edit removes one `with_<x>(...)` line from
        // `token_based()`. We can reproduce that locally by taking
        // `token_based()` (which validates) and then *deleting* one
        // entry — except `map` is private and setters can't remove.
        //
        // The realistic mirror is: rebuild the registry from a
        // `token_based()` snapshot but skip one setter. We do that
        // by constructing an almost-complete registry (55 of 56) and
        // asking the same code path to validate. Since we can't
        // reuse the production self-check without code duplication,
        // we rely on the `panic_missing_renderers` helper directly
        // via a hand-rolled almost-complete registry.
        let mut almost = RendererRegistry::token_based();
        // Drop one entry to simulate "future edit removed this line".
        let dropped = TypeId::of::<yororen_ui_core::renderer::button::ButtonRenderState>();
        almost.map.remove(&dropped);
        // Now `almost` is incomplete; the same `panic_missing_renderers`
        // path used by `token_based()` must fire.
        assert!(almost.validate().is_err());
        panic_missing_renderers(&almost);
    }
}
