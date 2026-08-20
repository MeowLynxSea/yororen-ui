//! Bundled theme JSON loaders and the WinUI install helper.
//!
//! The renderer crate ships two WinUI themes (light + dark) as
//! JSON files under `themes/`. Load them via [`winui_light`]
//! / [`winui_dark`], or pass any user-authored JSON to
//! `Theme::from_json`.
//!
//! [`install`] is the one-call bootstrap: it loads
//! `winui-light.json` (or `winui-dark.json` if
//! `WindowAppearance` is dark), populates the global theme, and
//! registers the 56 `WinUIXxxRenderer` impls against
//! the core `RendererRegistry`. App code calls it once at
//! startup, then `cx.theme()` /
//! `cx.renderer_arc::<markers::Button, dyn ButtonRenderer>()`
//! work everywhere.

use std::sync::Arc;

use gpui::{App, SharedString, WindowAppearance};

use yororen_ui_core::renderer::RendererContext;
use yororen_ui_core::renderer::markers;
use yororen_ui_core::theme::{Theme, install as install_theme};

use crate::renderers::{
    WinUIAvatarRenderer, WinUIBadgeRenderer, WinUIButtonGroupRenderer, WinUIButtonRenderer,
    WinUICardRenderer, WinUICheckboxRenderer, WinUIComboBoxRenderer, WinUIDisclosureRenderer,
    WinUIDividerRenderer, WinUIDropdownMenuRenderer, WinUIEmptyStateRenderer,
    WinUIFilePathInputRenderer, WinUIFocusRingRenderer, WinUIFormFieldRenderer, WinUIFormRenderer,
    WinUIGridViewRenderer,
    WinUIHeadingRenderer, WinUIIconButtonRenderer, WinUIIconRenderer, WinUIImageRenderer,
    WinUIKeybindingDisplayRenderer, WinUIKeybindingInputRenderer, WinUILabelRenderer,
    WinUIListItemRenderer, WinUIListboxRenderer, WinUIMenuRenderer, WinUIModalRenderer,
    WinUINotificationRenderer, WinUINumberInputRenderer, WinUIOverlayRenderer, WinUIPanelRenderer,
    WinUIPasswordInputRenderer, WinUIPopoverRenderer, WinUIProgressBarRenderer,
    WinUIRadioGroupRenderer, WinUIRadioRenderer, WinUISearchInputRenderer, WinUISelectRenderer,
    WinUIShortcutHintRenderer, WinUISkeletonRenderer, WinUISliderRenderer, WinUISpacerRenderer,
    WinUISplitButtonRenderer, WinUISwitchRenderer, WinUITableRenderer, WinUITagRenderer,
    WinUITextAreaRenderer, WinUITextInputRenderer, WinUITextRenderer, WinUIToastRenderer,
    WinUIToggleButtonRenderer, WinUITooltipRenderer, WinUITreeItemRenderer, WinUITreeRenderer,
    WinUIUniformVirtualListRenderer, WinUIVirtualListRenderer,
};

/// Load `themes/winui-light.json` as a `Theme`.
pub fn winui_light() -> Theme {
    Theme::from_json(include_str!("../themes/winui-light.json"))
        .expect("themes/winui-light.json is valid")
}

/// Load `themes/winui-dark.json` as a `Theme`.
pub fn winui_dark() -> Theme {
    Theme::from_json(include_str!("../themes/winui-dark.json"))
        .expect("themes/winui-dark.json is valid")
}

/// The WinUI UI font family from the active theme
/// (`tokens.typography.family_default`, e.g. "Segoe UI Variable"),
/// falling back to `system-ui`.
pub fn default_font(theme: &Theme) -> SharedString {
    theme
        .get_string("tokens.typography.family_default")
        .map(|s| s.to_string().into())
        .unwrap_or_else(|| SharedString::from("system-ui"))
}

/// WinUI TextBox field padding: `5px 6px 6px 10px` (top / right /
/// bottom / left) — the asymmetric padding the reference applies to
/// the field inside the 32px border box.
pub fn input_field_padding(theme: &Theme) -> yororen_ui_core::renderer::spec::Edges<gpui::Pixels> {
    use yororen_ui_core::renderer::spec::Edges;
    let get = |key: &str, fallback: f64| {
        gpui::px(
            theme
                .get_number(&format!("tokens.control.input.{key}"))
                .unwrap_or(fallback) as f32,
        )
    };
    Edges {
        top: get("padding_top", 5.0),
        right: get("padding_right", 6.0),
        bottom: get("padding_bottom", 6.0),
        left: get("padding_left", 10.0),
    }
}

/// Pick a WinUI theme based on OS appearance.
pub fn winui_for(appearance: WindowAppearance) -> Theme {
    match appearance {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => winui_dark(),
        WindowAppearance::Light | WindowAppearance::VibrantLight => winui_light(),
    }
}

/// One-call bootstrap. Picks a system theme by OS appearance,
/// installs the global `Theme`, and registers the 56 default
/// `WinUIXxxRenderer` impls against the core
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
    install_with(cx, winui_for(appearance));
}

/// Same as [`install`], but the caller provides the `Theme`.
/// Useful for tests and for apps that ship their own JSON
/// theme.
pub fn install_with(cx: &mut App, theme: Theme) {
    install_theme(cx, theme);
    register_winui_renderers(cx);
}

/// Register the 56 default `WinUIXxxRenderer` impls against
/// the core `RendererRegistry`. Public so a caller who already
/// installed the theme (e.g. for tests) can still wire up the
/// default look without re-installing the theme.
pub fn register_winui_renderers(cx: &mut App) {
    cx.register_renderer_arc::<markers::Button, dyn crate::renderers::ButtonRenderer>(Arc::new(
        WinUIButtonRenderer,
    ));
    cx.register_renderer_arc::<markers::ButtonGroup, dyn crate::renderers::ButtonGroupRenderer>(
        Arc::new(WinUIButtonGroupRenderer),
    );
    cx.register_renderer_arc::<markers::IconButton, dyn crate::renderers::IconButtonRenderer>(
        Arc::new(WinUIIconButtonRenderer),
    );
    cx.register_renderer_arc::<markers::ToggleButton, dyn crate::renderers::ToggleButtonRenderer>(
        Arc::new(WinUIToggleButtonRenderer),
    );
    cx.register_renderer_arc::<markers::Label, dyn crate::renderers::LabelRenderer>(Arc::new(
        WinUILabelRenderer,
    ));
    cx.register_renderer_arc::<markers::Heading, dyn crate::renderers::HeadingRenderer>(Arc::new(
        WinUIHeadingRenderer,
    ));
    cx.register_renderer_arc::<markers::Icon, dyn crate::renderers::IconRenderer>(Arc::new(
        WinUIIconRenderer,
    ));
    cx.register_renderer_arc::<markers::Text, dyn crate::renderers::TextRenderer>(Arc::new(
        WinUITextRenderer,
    ));
    cx.register_renderer_arc::<markers::Divider, dyn crate::renderers::DividerRenderer>(Arc::new(
        WinUIDividerRenderer,
    ));
    cx.register_renderer_arc::<markers::FocusRing, dyn crate::renderers::FocusRingRenderer>(
        Arc::new(WinUIFocusRingRenderer),
    );
    cx.register_renderer_arc::<markers::Badge, dyn crate::renderers::BadgeRenderer>(Arc::new(
        WinUIBadgeRenderer,
    ));
    cx.register_renderer_arc::<markers::Tag, dyn crate::renderers::TagRenderer>(Arc::new(
        WinUITagRenderer,
    ));
    cx.register_renderer_arc::<markers::ProgressBar, dyn crate::renderers::ProgressBarRenderer>(
        Arc::new(WinUIProgressBarRenderer),
    );
    cx.register_renderer_arc::<markers::Skeleton, dyn crate::renderers::SkeletonRenderer>(
        Arc::new(WinUISkeletonRenderer),
    );
    cx.register_renderer_arc::<markers::Slider, dyn crate::renderers::SliderRenderer>(Arc::new(
        WinUISliderRenderer,
    ));
    cx.register_renderer_arc::<markers::Tooltip, dyn crate::renderers::TooltipRenderer>(Arc::new(
        WinUITooltipRenderer,
    ));
    cx.register_renderer_arc::<markers::Avatar, dyn crate::renderers::AvatarRenderer>(Arc::new(
        WinUIAvatarRenderer,
    ));
    cx.register_renderer_arc::<markers::Switch, dyn crate::renderers::SwitchRenderer>(Arc::new(
        WinUISwitchRenderer,
    ));
    cx.register_renderer_arc::<markers::Checkbox, dyn crate::renderers::CheckboxRenderer>(
        Arc::new(WinUICheckboxRenderer),
    );
    cx.register_renderer_arc::<markers::Radio, dyn crate::renderers::RadioRenderer>(Arc::new(
        WinUIRadioRenderer,
    ));
    cx.register_renderer_arc::<markers::TextInput, dyn crate::renderers::TextInputRenderer>(
        Arc::new(WinUITextInputRenderer),
    );
    cx.register_renderer_arc::<markers::TextArea, dyn crate::renderers::TextAreaRenderer>(
        Arc::new(WinUITextAreaRenderer),
    );
    cx.register_renderer_arc::<markers::PasswordInput, dyn crate::renderers::PasswordInputRenderer>(
        Arc::new(WinUIPasswordInputRenderer),
    );
    cx.register_renderer_arc::<markers::NumberInput, dyn crate::renderers::NumberInputRenderer>(
        Arc::new(WinUINumberInputRenderer),
    );
    cx.register_renderer_arc::<markers::FilePathInput, dyn crate::renderers::FilePathInputRenderer>(
        Arc::new(WinUIFilePathInputRenderer),
    );
    cx.register_renderer_arc::<markers::SearchInput, dyn crate::renderers::SearchInputRenderer>(
        Arc::new(WinUISearchInputRenderer),
    );
    cx.register_renderer_arc::<markers::Select, dyn crate::renderers::SelectRenderer>(Arc::new(
        WinUISelectRenderer,
    ));
    cx.register_renderer_arc::<markers::ComboBox, dyn crate::renderers::ComboBoxRenderer>(
        Arc::new(WinUIComboBoxRenderer),
    );
    cx.register_renderer_arc::<markers::Modal, dyn crate::renderers::ModalRenderer>(Arc::new(
        WinUIModalRenderer,
    ));
    cx.register_renderer_arc::<markers::Popover, dyn crate::renderers::PopoverRenderer>(Arc::new(
        WinUIPopoverRenderer,
    ));
    cx.register_renderer_arc::<markers::DropdownMenu, dyn crate::renderers::DropdownMenuRenderer>(
        Arc::new(WinUIDropdownMenuRenderer),
    );
    cx.register_renderer_arc::<markers::Disclosure, dyn crate::renderers::DisclosureRenderer>(
        Arc::new(WinUIDisclosureRenderer),
    );
    cx.register_renderer_arc::<markers::Toast, dyn crate::renderers::ToastRenderer>(Arc::new(
        WinUIToastRenderer,
    ));
    cx.register_renderer_arc::<markers::Notification, dyn crate::renderers::NotificationRenderer>(
        Arc::new(WinUINotificationRenderer),
    );
    cx.register_renderer_arc::<markers::Panel, dyn crate::renderers::PanelRenderer>(Arc::new(
        WinUIPanelRenderer,
    ));
    cx.register_renderer_arc::<markers::Card, dyn crate::renderers::CardRenderer>(Arc::new(
        WinUICardRenderer,
    ));
    cx.register_renderer_arc::<markers::Form, dyn crate::renderers::FormRenderer>(Arc::new(
        WinUIFormRenderer,
    ));
    cx.register_renderer_arc::<markers::FormField, dyn crate::renderers::FormFieldRenderer>(
        Arc::new(WinUIFormFieldRenderer),
    );
    cx.register_renderer_arc::<markers::GridView, dyn crate::renderers::GridViewRenderer>(
        Arc::new(WinUIGridViewRenderer),
    );
    cx.register_renderer_arc::<markers::ListItem, dyn crate::renderers::ListItemRenderer>(
        Arc::new(WinUIListItemRenderer),
    );
    cx.register_renderer_arc::<markers::Listbox, dyn crate::renderers::ListboxRenderer>(Arc::new(
        WinUIListboxRenderer,
    ));
    cx.register_renderer_arc::<markers::Menu, dyn crate::renderers::MenuRenderer>(Arc::new(
        WinUIMenuRenderer,
    ));
    cx.register_renderer_arc::<markers::Overlay, dyn crate::renderers::OverlayRenderer>(Arc::new(
        WinUIOverlayRenderer,
    ));
    cx.register_renderer_arc::<markers::RadioGroup, dyn crate::renderers::RadioGroupRenderer>(
        Arc::new(WinUIRadioGroupRenderer),
    );
    cx.register_renderer_arc::<markers::Spacer, dyn crate::renderers::SpacerRenderer>(Arc::new(
        WinUISpacerRenderer,
    ));
    cx.register_renderer_arc::<markers::Table, dyn crate::renderers::TableRenderer>(Arc::new(
        WinUITableRenderer,
    ));
    cx.register_renderer_arc::<markers::Tree, dyn crate::renderers::TreeRenderer>(Arc::new(
        WinUITreeRenderer,
    ));
    cx.register_renderer_arc::<markers::TreeItem, dyn crate::renderers::TreeItemRenderer>(
        Arc::new(WinUITreeItemRenderer),
    );
    cx.register_renderer_arc::<markers::VirtualList, dyn crate::renderers::VirtualListRenderer>(
        Arc::new(WinUIVirtualListRenderer),
    );
    cx.register_renderer_arc::<markers::UniformVirtualList, dyn crate::renderers::UniformVirtualListRenderer>(
        Arc::new(WinUIUniformVirtualListRenderer),
    );
    cx.register_renderer_arc::<markers::KeybindingInput, dyn crate::renderers::KeybindingInputRenderer>(
        Arc::new(WinUIKeybindingInputRenderer),
    );
    cx.register_renderer_arc::<markers::SplitButton, dyn crate::renderers::SplitButtonRenderer>(
        Arc::new(WinUISplitButtonRenderer),
    );
    cx.register_renderer_arc::<markers::EmptyState, dyn crate::renderers::EmptyStateRenderer>(
        Arc::new(WinUIEmptyStateRenderer),
    );
    cx.register_renderer_arc::<markers::Image, dyn crate::renderers::ImageRenderer>(Arc::new(
        WinUIImageRenderer,
    ));
    cx.register_renderer_arc::<markers::KeybindingDisplay, dyn crate::renderers::KeybindingDisplayRenderer>(
        Arc::new(WinUIKeybindingDisplayRenderer),
    );
    cx.register_renderer_arc::<markers::ShortcutHint, dyn crate::renderers::ShortcutHintRenderer>(
        Arc::new(WinUIShortcutHintRenderer),
    );
}
