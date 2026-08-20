//! Component renderer traits, one per component. The
//! `ButtonRenderer` entry is the reference example; the other 37
//! follow the same pattern.
//!
//! **As of v0.3.6** the `XxxRenderer` trait + `XxxRenderState`
//! struct live in `yororen-ui-core` (so headless `XxxProps::render`
//! can call them). This module re-exports them and provides the
//! `TokenXxxRenderer` default impls.

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

pub use avatar::TokenAvatarRenderer;
pub use badge::TokenBadgeRenderer;
pub use button::TokenButtonRenderer;
pub use button_group::TokenButtonGroupRenderer;
pub use card::TokenCardRenderer;
pub use checkbox::TokenCheckboxRenderer;
pub use combo_box::TokenComboBoxRenderer;
pub use disclosure::TokenDisclosureRenderer;
pub use divider::TokenDividerRenderer;
pub use dropdown_menu::TokenDropdownMenuRenderer;
pub use empty_state::TokenEmptyStateRenderer;
pub use file_path_input::TokenFilePathInputRenderer;
pub use focus_ring::TokenFocusRingRenderer;
pub use form::TokenFormRenderer;
pub use form_field::TokenFormFieldRenderer;
pub use grid_view::TokenGridViewRenderer;
pub use heading::TokenHeadingRenderer;
pub use icon::TokenIconRenderer;
pub use icon_button::TokenIconButtonRenderer;
pub use image::TokenImageRenderer;
pub use keybinding_display::TokenKeybindingDisplayRenderer;
pub use keybinding_input::TokenKeybindingInputRenderer;
pub use label::TokenLabelRenderer;
pub use list_item::TokenListItemRenderer;
pub use listbox::TokenListboxRenderer;
pub use menu::TokenMenuRenderer;
pub use modal::TokenModalRenderer;
pub use notification::TokenNotificationRenderer;
pub use number_input::TokenNumberInputRenderer;
pub use overlay::TokenOverlayRenderer;
pub use panel::TokenPanelRenderer;
pub use password_input::TokenPasswordInputRenderer;
pub use popover::TokenPopoverRenderer;
pub use progress::TokenProgressBarRenderer;
pub use radio::TokenRadioRenderer;
pub use radio_group::TokenRadioGroupRenderer;
pub use registry::RendererRegistry;
pub use search_input::TokenSearchInputRenderer;
pub use select::TokenSelectRenderer;
pub use shortcut_hint::TokenShortcutHintRenderer;
pub use skeleton::TokenSkeletonRenderer;
pub use slider::TokenSliderRenderer;
pub use spacer::TokenSpacerRenderer;
pub use split_button::TokenSplitButtonRenderer;
pub use switch::TokenSwitchRenderer;
pub use table::TokenTableRenderer;
pub use tag::TokenTagRenderer;
pub use text::TokenTextRenderer;
pub use text_area::TokenTextAreaRenderer;
pub use text_input::TokenTextInputRenderer;
pub use toast::TokenToastRenderer;
pub use toggle_button::TokenToggleButtonRenderer;
pub use tooltip::TokenTooltipRenderer;
pub use tree::TokenTreeRenderer;
pub use tree_item::TokenTreeItemRenderer;
pub use uniform_virtual_list::TokenUniformVirtualListRenderer;
pub use virtual_list::TokenVirtualListRenderer;
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
