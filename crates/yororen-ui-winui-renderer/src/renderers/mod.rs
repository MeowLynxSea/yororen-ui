//! Component renderer traits, one per component. The
//! `ButtonRenderer` entry is the reference example; the other 37
//! follow the same pattern.
//!
//! **As of v0.3.6** the `XxxRenderer` trait + `XxxRenderState`
//! struct live in `yororen-ui-core` (so headless `XxxProps::render`
//! can call them). This module re-exports them and provides the
//! `WinUIXxxRenderer` default impls.

pub mod avatar;
pub mod badge;
pub mod button;
pub mod button_group;
pub mod card;
pub mod checkbox;
pub mod combo_box;
pub mod disclosure;
pub mod divider;
pub mod dropdown_menu;
pub mod empty_state;
pub mod file_path_input;
pub mod focus_ring;
pub mod form;
pub mod form_field;
pub mod grid_view;
pub mod heading;
pub mod icon;
pub mod icon_button;
pub mod image;
pub mod keybinding_display;
pub mod keybinding_input;
pub mod label;
pub mod list_item;
pub mod listbox;
pub mod menu;
pub mod modal;
pub mod notification;
pub mod number_input;
pub mod overlay;
pub mod panel;
pub mod password_input;
pub mod popover;
pub mod progress;
pub mod radio;
pub mod radio_group;
pub mod registry;
pub mod search_input;
pub mod select;
pub mod shortcut_hint;
pub mod skeleton;
pub mod slider;
pub mod spacer;
pub mod split_button;
pub mod switch;
pub mod table;
pub mod tag;
pub mod text;
pub mod text_area;
pub mod text_input;
pub mod toast;
pub mod toggle_button;
pub mod tooltip;
pub mod tree;
pub mod tree_item;
pub mod uniform_virtual_list;
pub mod variant;
pub mod virtual_list;

pub use avatar::WinUIAvatarRenderer;
pub use badge::WinUIBadgeRenderer;
pub use button::WinUIButtonRenderer;
pub use button_group::WinUIButtonGroupRenderer;
pub use card::WinUICardRenderer;
pub use checkbox::WinUICheckboxRenderer;
pub use combo_box::WinUIComboBoxRenderer;
pub use disclosure::WinUIDisclosureRenderer;
pub use divider::WinUIDividerRenderer;
pub use dropdown_menu::WinUIDropdownMenuRenderer;
pub use empty_state::WinUIEmptyStateRenderer;
pub use file_path_input::WinUIFilePathInputRenderer;
pub use focus_ring::WinUIFocusRingRenderer;
pub use form::WinUIFormRenderer;
pub use form_field::WinUIFormFieldRenderer;
pub use grid_view::WinUIGridViewRenderer;
pub use heading::WinUIHeadingRenderer;
pub use icon::WinUIIconRenderer;
pub use icon_button::WinUIIconButtonRenderer;
pub use image::WinUIImageRenderer;
pub use keybinding_display::WinUIKeybindingDisplayRenderer;
pub use keybinding_input::WinUIKeybindingInputRenderer;
pub use label::WinUILabelRenderer;
pub use list_item::WinUIListItemRenderer;
pub use listbox::WinUIListboxRenderer;
pub use menu::WinUIMenuRenderer;
pub use modal::WinUIModalRenderer;
pub use notification::WinUINotificationRenderer;
pub use number_input::WinUINumberInputRenderer;
pub use overlay::WinUIOverlayRenderer;
pub use panel::WinUIPanelRenderer;
pub use password_input::WinUIPasswordInputRenderer;
pub use popover::WinUIPopoverRenderer;
pub use progress::WinUIProgressBarRenderer;
pub use radio::WinUIRadioRenderer;
pub use radio_group::WinUIRadioGroupRenderer;
pub use registry::RendererRegistry;
pub use search_input::WinUISearchInputRenderer;
pub use select::WinUISelectRenderer;
pub use shortcut_hint::WinUIShortcutHintRenderer;
pub use skeleton::WinUISkeletonRenderer;
pub use slider::WinUISliderRenderer;
pub use spacer::WinUISpacerRenderer;
pub use split_button::WinUISplitButtonRenderer;
pub use switch::WinUISwitchRenderer;
pub use table::WinUITableRenderer;
pub use tag::WinUITagRenderer;
pub use text::WinUITextRenderer;
pub use text_area::WinUITextAreaRenderer;
pub use text_input::WinUITextInputRenderer;
pub use toast::WinUIToastRenderer;
pub use toggle_button::WinUIToggleButtonRenderer;
pub use tooltip::WinUITooltipRenderer;
pub use tree::WinUITreeRenderer;
pub use tree_item::WinUITreeItemRenderer;
pub use uniform_virtual_list::WinUIUniformVirtualListRenderer;
pub use virtual_list::WinUIVirtualListRenderer;
pub use yororen_ui_core::renderer::avatar::{AvatarRenderState, AvatarRenderer};
pub use yororen_ui_core::renderer::badge::{BadgeRenderState, BadgeRenderer};
pub use yororen_ui_core::renderer::button::{ButtonRenderState, ButtonRenderer};
pub use yororen_ui_core::renderer::button_group::{ButtonGroupRenderState, ButtonGroupRenderer};
pub use yororen_ui_core::renderer::card::{CardRenderState, CardRenderer};
pub use yororen_ui_core::renderer::checkbox::{CheckboxRenderState, CheckboxRenderer};
pub use yororen_ui_core::renderer::combo_box::{ComboBoxRenderState, ComboBoxRenderer};
pub use yororen_ui_core::renderer::disclosure::{DisclosureRenderState, DisclosureRenderer};
pub use yororen_ui_core::renderer::divider::{DividerRenderState, DividerRenderer};
pub use yororen_ui_core::renderer::dropdown_menu::{DropdownMenuRenderState, DropdownMenuRenderer};
pub use yororen_ui_core::renderer::empty_state::{EmptyStateRenderState, EmptyStateRenderer};
pub use yororen_ui_core::renderer::file_path_input::{
    FilePathInputRenderState, FilePathInputRenderer,
};
pub use yororen_ui_core::renderer::focus_ring::{FocusRingRenderState, FocusRingRenderer};
pub use yororen_ui_core::renderer::form::{FormRenderState, FormRenderer};
pub use yororen_ui_core::renderer::form_field::{FormFieldRenderState, FormFieldRenderer};
pub use yororen_ui_core::renderer::grid_view::{GridViewRenderState, GridViewRenderer};
pub use yororen_ui_core::renderer::heading::{HeadingRenderState, HeadingRenderer};
pub use yororen_ui_core::renderer::icon::{IconRenderState, IconRenderer};
pub use yororen_ui_core::renderer::icon_button::{IconButtonRenderState, IconButtonRenderer};
pub use yororen_ui_core::renderer::image::{ImageRenderState, ImageRenderer};
pub use yororen_ui_core::renderer::keybinding_display::{
    KeybindingDisplayRenderState, KeybindingDisplayRenderer,
};
pub use yororen_ui_core::renderer::keybinding_input::{
    KeybindingInputRenderState, KeybindingInputRenderer,
};
pub use yororen_ui_core::renderer::label::{LabelRenderState, LabelRenderer};
pub use yororen_ui_core::renderer::list_item::{ListItemRenderState, ListItemRenderer};
pub use yororen_ui_core::renderer::listbox::{ListboxRenderState, ListboxRenderer};
pub use yororen_ui_core::renderer::menu::{MenuRenderState, MenuRenderer};
pub use yororen_ui_core::renderer::modal::{ModalRenderState, ModalRenderer};
pub use yororen_ui_core::renderer::notification::{NotificationRenderState, NotificationRenderer};
pub use yororen_ui_core::renderer::number_input::{NumberInputRenderState, NumberInputRenderer};
pub use yororen_ui_core::renderer::overlay::{OverlayRenderState, OverlayRenderer};
pub use yororen_ui_core::renderer::panel::{PanelRenderState, PanelRenderer};
pub use yororen_ui_core::renderer::password_input::{
    PasswordInputRenderState, PasswordInputRenderer,
};
pub use yororen_ui_core::renderer::popover::{PopoverRenderState, PopoverRenderer};
pub use yororen_ui_core::renderer::progress::{ProgressBarRenderState, ProgressBarRenderer};
pub use yororen_ui_core::renderer::radio::{RadioRenderState, RadioRenderer};
pub use yororen_ui_core::renderer::radio_group::{RadioGroupRenderState, RadioGroupRenderer};
pub use yororen_ui_core::renderer::search_input::{SearchInputRenderState, SearchInputRenderer};
pub use yororen_ui_core::renderer::select::{SelectRenderState, SelectRenderer};
pub use yororen_ui_core::renderer::shortcut_hint::{ShortcutHintRenderState, ShortcutHintRenderer};
pub use yororen_ui_core::renderer::skeleton::{SkeletonRenderState, SkeletonRenderer};
pub use yororen_ui_core::renderer::slider::{SliderRenderState, SliderRenderer};
pub use yororen_ui_core::renderer::spacer::{SpacerRenderState, SpacerRenderer};
pub use yororen_ui_core::renderer::split_button::{SplitButtonRenderState, SplitButtonRenderer};
pub use yororen_ui_core::renderer::switch::{SwitchRenderState, SwitchRenderer};
pub use yororen_ui_core::renderer::table::{TableRenderState, TableRenderer};
pub use yororen_ui_core::renderer::tag::{TagRenderState, TagRenderer};
pub use yororen_ui_core::renderer::text::{TextRenderState, TextRenderer};
pub use yororen_ui_core::renderer::text_area::{TextAreaRenderState, TextAreaRenderer};
pub use yororen_ui_core::renderer::text_input::{TextInputRenderState, TextInputRenderer};
pub use yororen_ui_core::renderer::toast::{ToastRenderState, ToastRenderer};
pub use yororen_ui_core::renderer::toggle_button::{ToggleButtonRenderState, ToggleButtonRenderer};
pub use yororen_ui_core::renderer::tooltip::{TooltipRenderState, TooltipRenderer};
pub use yororen_ui_core::renderer::tree::{TreeRenderState, TreeRenderer};
pub use yororen_ui_core::renderer::tree_item::{TreeItemRenderState, TreeItemRenderer};
pub use yororen_ui_core::renderer::uniform_virtual_list::{
    UniformVirtualListRenderState, UniformVirtualListRenderer,
};
pub use yororen_ui_core::renderer::variant::ActionVariantKind;
pub use yororen_ui_core::renderer::variant::{
    BuiltinVariantKey, ButtonVariant, GlobalVariantRegistry, TokenVariantStyle, VariantKey,
    VariantRegistry, VariantState, VariantStyle, variant_compose,
};
pub use yororen_ui_core::renderer::virtual_list::{VirtualListRenderState, VirtualListRenderer};
