//! Bundled theme JSON loaders and the default install helper.
//!
//! The renderer crate ships two system themes (light + dark) as
//! JSON files under `themes/`. Load them via [`system_light`]
//! / [`system_dark`], or pass any user-authored JSON to
//! `Theme::from_json`.
//!
//! [`install`] is the one-call bootstrap: it loads
//! `system-light.json` (or `system-dark.json` if
//! `WindowAppearance` is dark), populates the global theme, and
//! registers the 56 default `TokenXxxRenderer` impls against
//! the core `RendererRegistry`. App code calls it once at
//! startup, then `cx.theme()` /
//! `cx.renderer_arc::<markers::Button, dyn ButtonRenderer>()`
//! work everywhere.

use std::sync::Arc;

use gpui::{App, WindowAppearance};

use yororen_ui_core::renderer::RendererContext;
use yororen_ui_core::renderer::markers;
use yororen_ui_core::theme::{Theme, install as install_theme};

use crate::renderers::{
    TokenAvatarRenderer, TokenBadgeRenderer, TokenButtonGroupRenderer, TokenButtonRenderer,
    TokenCardRenderer, TokenCheckboxRenderer, TokenComboBoxRenderer, TokenDisclosureRenderer,
    TokenDividerRenderer, TokenDropdownMenuRenderer, TokenEmptyStateRenderer,
    TokenFilePathInputRenderer, TokenFocusRingRenderer, TokenFormFieldRenderer, TokenFormRenderer,
    TokenGridViewRenderer, TokenHeadingRenderer, TokenIconButtonRenderer, TokenIconRenderer,
    TokenImageRenderer,
    TokenKeybindingDisplayRenderer, TokenKeybindingInputRenderer, TokenLabelRenderer,
    TokenListItemRenderer, TokenListboxRenderer, TokenMenuRenderer, TokenModalRenderer,
    TokenNotificationRenderer, TokenNumberInputRenderer, TokenOverlayRenderer, TokenPanelRenderer,
    TokenPasswordInputRenderer, TokenPopoverRenderer, TokenProgressBarRenderer,
    TokenRadioGroupRenderer, TokenRadioRenderer, TokenSearchInputRenderer, TokenSelectRenderer,
    TokenShortcutHintRenderer, TokenSkeletonRenderer, TokenSliderRenderer, TokenSpacerRenderer,
    TokenSplitButtonRenderer, TokenSwitchRenderer, TokenTableRenderer, TokenTagRenderer,
    TokenTextAreaRenderer, TokenTextInputRenderer, TokenTextRenderer, TokenToastRenderer,
    TokenToggleButtonRenderer, TokenTooltipRenderer, TokenTreeItemRenderer, TokenTreeRenderer,
    TokenUniformVirtualListRenderer, TokenVirtualListRenderer,
};

/// Load `themes/system-light.json` as a `Theme`.
pub fn system_light() -> Theme {
    Theme::from_json(include_str!("../themes/system-light.json"))
        .expect("themes/system-light.json is valid")
}

/// Load `themes/system-dark.json` as a `Theme`.
pub fn system_dark() -> Theme {
    Theme::from_json(include_str!("../themes/system-dark.json"))
        .expect("themes/system-dark.json is valid")
}

/// Pick a system theme based on OS appearance.
pub fn system_for(appearance: WindowAppearance) -> Theme {
    match appearance {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => system_dark(),
        WindowAppearance::Light | WindowAppearance::VibrantLight => system_light(),
    }
}

/// One-call bootstrap. Picks a system theme by OS appearance,
/// installs the global `Theme`, and registers the 56 default
/// `TokenXxxRenderer` impls against the core
/// `RendererRegistry`. Call this once at app boot, before any
/// component renders.
///
/// ```ignore
/// app.run(|cx| {
///     yororen_ui_renderer::install(cx, window.appearance());
///     // ... open windows
/// });
/// ```
pub fn install(cx: &mut App, appearance: WindowAppearance) {
    install_with(cx, system_for(appearance));
}

/// Same as [`install`], but the caller provides the `Theme`.
/// Useful for tests and for apps that ship their own JSON
/// theme.
pub fn install_with(cx: &mut App, theme: Theme) {
    install_theme(cx, theme);
    register_default_renderers(cx);
}

/// Register the 56 default `TokenXxxRenderer` impls against
/// the core `RendererRegistry`. Public so a caller who already
/// installed the theme (e.g. for tests) can still wire up the
/// default look without re-installing the theme.
pub fn register_default_renderers(cx: &mut App) {
    cx.register_renderer_arc::<markers::Button, dyn crate::renderers::ButtonRenderer>(Arc::new(
        TokenButtonRenderer,
    ));
    cx.register_renderer_arc::<markers::ButtonGroup, dyn crate::renderers::ButtonGroupRenderer>(
        Arc::new(TokenButtonGroupRenderer),
    );
    cx.register_renderer_arc::<markers::IconButton, dyn crate::renderers::IconButtonRenderer>(
        Arc::new(TokenIconButtonRenderer),
    );
    cx.register_renderer_arc::<markers::ToggleButton, dyn crate::renderers::ToggleButtonRenderer>(
        Arc::new(TokenToggleButtonRenderer),
    );
    cx.register_renderer_arc::<markers::Label, dyn crate::renderers::LabelRenderer>(Arc::new(
        TokenLabelRenderer,
    ));
    cx.register_renderer_arc::<markers::Heading, dyn crate::renderers::HeadingRenderer>(Arc::new(
        TokenHeadingRenderer,
    ));
    cx.register_renderer_arc::<markers::Icon, dyn crate::renderers::IconRenderer>(Arc::new(
        TokenIconRenderer,
    ));
    cx.register_renderer_arc::<markers::Text, dyn crate::renderers::TextRenderer>(Arc::new(
        TokenTextRenderer,
    ));
    cx.register_renderer_arc::<markers::Divider, dyn crate::renderers::DividerRenderer>(Arc::new(
        TokenDividerRenderer,
    ));
    cx.register_renderer_arc::<markers::FocusRing, dyn crate::renderers::FocusRingRenderer>(
        Arc::new(TokenFocusRingRenderer),
    );
    cx.register_renderer_arc::<markers::Badge, dyn crate::renderers::BadgeRenderer>(Arc::new(
        TokenBadgeRenderer,
    ));
    cx.register_renderer_arc::<markers::Tag, dyn crate::renderers::TagRenderer>(Arc::new(
        TokenTagRenderer,
    ));
    cx.register_renderer_arc::<markers::ProgressBar, dyn crate::renderers::ProgressBarRenderer>(
        Arc::new(TokenProgressBarRenderer),
    );
    cx.register_renderer_arc::<markers::Skeleton, dyn crate::renderers::SkeletonRenderer>(
        Arc::new(TokenSkeletonRenderer),
    );
    cx.register_renderer_arc::<markers::Slider, dyn crate::renderers::SliderRenderer>(Arc::new(
        TokenSliderRenderer,
    ));
    cx.register_renderer_arc::<markers::Tooltip, dyn crate::renderers::TooltipRenderer>(Arc::new(
        TokenTooltipRenderer,
    ));
    cx.register_renderer_arc::<markers::Avatar, dyn crate::renderers::AvatarRenderer>(Arc::new(
        TokenAvatarRenderer,
    ));
    cx.register_renderer_arc::<markers::Switch, dyn crate::renderers::SwitchRenderer>(Arc::new(
        TokenSwitchRenderer,
    ));
    cx.register_renderer_arc::<markers::Checkbox, dyn crate::renderers::CheckboxRenderer>(
        Arc::new(TokenCheckboxRenderer),
    );
    cx.register_renderer_arc::<markers::Radio, dyn crate::renderers::RadioRenderer>(Arc::new(
        TokenRadioRenderer,
    ));
    cx.register_renderer_arc::<markers::TextInput, dyn crate::renderers::TextInputRenderer>(
        Arc::new(TokenTextInputRenderer),
    );
    cx.register_renderer_arc::<markers::TextArea, dyn crate::renderers::TextAreaRenderer>(
        Arc::new(TokenTextAreaRenderer),
    );
    cx.register_renderer_arc::<markers::PasswordInput, dyn crate::renderers::PasswordInputRenderer>(
        Arc::new(TokenPasswordInputRenderer),
    );
    cx.register_renderer_arc::<markers::NumberInput, dyn crate::renderers::NumberInputRenderer>(
        Arc::new(TokenNumberInputRenderer),
    );
    cx.register_renderer_arc::<markers::FilePathInput, dyn crate::renderers::FilePathInputRenderer>(
        Arc::new(TokenFilePathInputRenderer),
    );
    cx.register_renderer_arc::<markers::SearchInput, dyn crate::renderers::SearchInputRenderer>(
        Arc::new(TokenSearchInputRenderer),
    );
    cx.register_renderer_arc::<markers::Select, dyn crate::renderers::SelectRenderer>(Arc::new(
        TokenSelectRenderer,
    ));
    cx.register_renderer_arc::<markers::ComboBox, dyn crate::renderers::ComboBoxRenderer>(
        Arc::new(TokenComboBoxRenderer),
    );
    cx.register_renderer_arc::<markers::Modal, dyn crate::renderers::ModalRenderer>(Arc::new(
        TokenModalRenderer,
    ));
    cx.register_renderer_arc::<markers::Popover, dyn crate::renderers::PopoverRenderer>(Arc::new(
        TokenPopoverRenderer,
    ));
    cx.register_renderer_arc::<markers::DropdownMenu, dyn crate::renderers::DropdownMenuRenderer>(
        Arc::new(TokenDropdownMenuRenderer),
    );
    cx.register_renderer_arc::<markers::Disclosure, dyn crate::renderers::DisclosureRenderer>(
        Arc::new(TokenDisclosureRenderer),
    );
    cx.register_renderer_arc::<markers::Toast, dyn crate::renderers::ToastRenderer>(Arc::new(
        TokenToastRenderer,
    ));
    cx.register_renderer_arc::<markers::Notification, dyn crate::renderers::NotificationRenderer>(
        Arc::new(TokenNotificationRenderer),
    );
    cx.register_renderer_arc::<markers::Panel, dyn crate::renderers::PanelRenderer>(Arc::new(
        TokenPanelRenderer,
    ));
    cx.register_renderer_arc::<markers::Card, dyn crate::renderers::CardRenderer>(Arc::new(
        TokenCardRenderer,
    ));
    cx.register_renderer_arc::<markers::Form, dyn crate::renderers::FormRenderer>(Arc::new(
        TokenFormRenderer,
    ));
    cx.register_renderer_arc::<markers::FormField, dyn crate::renderers::FormFieldRenderer>(
        Arc::new(TokenFormFieldRenderer),
    );
    cx.register_renderer_arc::<markers::GridView, dyn crate::renderers::GridViewRenderer>(
        Arc::new(TokenGridViewRenderer),
    );
    cx.register_renderer_arc::<markers::ListItem, dyn crate::renderers::ListItemRenderer>(
        Arc::new(TokenListItemRenderer),
    );
    cx.register_renderer_arc::<markers::Listbox, dyn crate::renderers::ListboxRenderer>(Arc::new(
        TokenListboxRenderer,
    ));
    cx.register_renderer_arc::<markers::Menu, dyn crate::renderers::MenuRenderer>(Arc::new(
        TokenMenuRenderer,
    ));
    cx.register_renderer_arc::<markers::Overlay, dyn crate::renderers::OverlayRenderer>(Arc::new(
        TokenOverlayRenderer,
    ));
    cx.register_renderer_arc::<markers::RadioGroup, dyn crate::renderers::RadioGroupRenderer>(
        Arc::new(TokenRadioGroupRenderer),
    );
    cx.register_renderer_arc::<markers::Spacer, dyn crate::renderers::SpacerRenderer>(Arc::new(
        TokenSpacerRenderer,
    ));
    cx.register_renderer_arc::<markers::Table, dyn crate::renderers::TableRenderer>(Arc::new(
        TokenTableRenderer,
    ));
    cx.register_renderer_arc::<markers::Tree, dyn crate::renderers::TreeRenderer>(Arc::new(
        TokenTreeRenderer,
    ));
    cx.register_renderer_arc::<markers::TreeItem, dyn crate::renderers::TreeItemRenderer>(
        Arc::new(TokenTreeItemRenderer),
    );
    cx.register_renderer_arc::<markers::VirtualList, dyn crate::renderers::VirtualListRenderer>(
        Arc::new(TokenVirtualListRenderer),
    );
    cx.register_renderer_arc::<markers::UniformVirtualList, dyn crate::renderers::UniformVirtualListRenderer>(
        Arc::new(TokenUniformVirtualListRenderer),
    );
    cx.register_renderer_arc::<markers::KeybindingInput, dyn crate::renderers::KeybindingInputRenderer>(
        Arc::new(TokenKeybindingInputRenderer),
    );
    cx.register_renderer_arc::<markers::SplitButton, dyn crate::renderers::SplitButtonRenderer>(
        Arc::new(TokenSplitButtonRenderer),
    );
    cx.register_renderer_arc::<markers::EmptyState, dyn crate::renderers::EmptyStateRenderer>(
        Arc::new(TokenEmptyStateRenderer),
    );
    cx.register_renderer_arc::<markers::Image, dyn crate::renderers::ImageRenderer>(Arc::new(
        TokenImageRenderer,
    ));
    cx.register_renderer_arc::<markers::KeybindingDisplay, dyn crate::renderers::KeybindingDisplayRenderer>(
        Arc::new(TokenKeybindingDisplayRenderer),
    );
    cx.register_renderer_arc::<markers::ShortcutHint, dyn crate::renderers::ShortcutHintRenderer>(
        Arc::new(TokenShortcutHintRenderer),
    );
}
