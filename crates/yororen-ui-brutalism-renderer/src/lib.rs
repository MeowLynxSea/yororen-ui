//! Neo-Brutalism style renderer for yororen-ui.
//!
//! Implements all 55 `XxxRenderer` traits with sharp corners, thick
//! black borders, hard offset shadows, and monospace typography.
//!
//! Two bundled themes: `brutalism-light.json` and
//! `brutalism-dark.json`. Use [`install_with_default_theme`] to
//! get the light theme, or [`install`] to pick light/dark by
//! system appearance.
//!
//! ```ignore
//! use yororen_ui_brutalism_renderer as brutalism;
//! brutalism::install(cx);
//! ```

// The crate is built up across several commits; until every
// `XxxRenderer` is wired in, individual helpers and constants
// will appear unused.
#![allow(dead_code)]

mod style;

pub mod renderers;

use std::sync::Arc;

use gpui::{App, WindowAppearance};
use yororen_ui_core::renderer::{RendererContext, markers as m};
use yororen_ui_core::theme::{Theme, install as install_theme};

use yororen_ui_core::renderer::button_group::ButtonGroupRenderer;
use yororen_ui_core::renderer::form_field::FormFieldRenderer;
use yororen_ui_core::renderer::grid_view::GridViewRenderer;
use yororen_ui_core::renderer::icon::IconRenderer as CoreIconRenderer;
use yororen_ui_core::renderer::image::ImageRenderer;
use yororen_ui_core::renderer::keybinding_display::KeybindingDisplayRenderer;
use yororen_ui_core::renderer::listbox::ListboxRenderer;
use yororen_ui_core::renderer::menu::MenuRenderer;
use yororen_ui_core::renderer::overlay::OverlayRenderer;
use yororen_ui_core::renderer::radio_group::RadioGroupRenderer;
use yororen_ui_core::renderer::shortcut_hint::ShortcutHintRenderer;
use yororen_ui_core::renderer::slider::SliderRenderer;
use yororen_ui_core::renderer::spacer::SpacerRenderer;
use yororen_ui_core::renderer::table::TableRenderer;
use yororen_ui_core::renderer::text::TextRenderer;
use yororen_ui_core::renderer::tree::TreeRenderer;
use yororen_ui_default_renderer::renderers::*;

use crate::renderers::{
    actions::{
        BrutalButtonGroupRenderer, BrutalButtonRenderer, BrutalIconButtonRenderer,
        BrutalSplitButtonRenderer, BrutalToggleButtonRenderer,
    },
    controls::{
        BrutalCheckboxRenderer, BrutalRadioGroupRenderer, BrutalRadioRenderer,
        BrutalSliderRenderer, BrutalSwitchRenderer,
    },
    display::{
        BrutalBadgeRenderer, BrutalDividerRenderer, BrutalEmptyStateRenderer,
        BrutalFocusRingRenderer, BrutalHeadingRenderer, BrutalIconRenderer,
        BrutalKeybindingDisplayRenderer, BrutalLabelRenderer, BrutalProgressBarRenderer,
        BrutalShortcutHintRenderer, BrutalSkeletonRenderer, BrutalSpacerRenderer,
        BrutalTagRenderer, BrutalTextRenderer,
    },
    inputs::{
        BrutalComboBoxRenderer, BrutalFilePathInputRenderer, BrutalKeybindingInputRenderer,
        BrutalNumberInputRenderer, BrutalPasswordInputRenderer, BrutalSearchInputRenderer,
        BrutalSelectRenderer, BrutalTextAreaRenderer, BrutalTextInputRenderer,
    },
    lists::{
        BrutalFormFieldRenderer, BrutalFormRenderer, BrutalGridViewRenderer, BrutalListItemRenderer,
        BrutalListboxRenderer, BrutalTableRenderer, BrutalTreeItemRenderer, BrutalTreeRenderer,
        BrutalUniformVirtualListRenderer, BrutalVirtualListRenderer,
    },
    notifications::{BrutalNotificationRenderer, BrutalToastRenderer},
    overlays::{
        BrutalDisclosureRenderer, BrutalDropdownMenuRenderer, BrutalMenuRenderer,
        BrutalModalRenderer, BrutalOverlayRenderer, BrutalPopoverRenderer,
    },
    surfaces::{
        BrutalAvatarRenderer, BrutalCardRenderer, BrutalImageRenderer, BrutalPanelRenderer,
        BrutalTooltipRenderer,
    },
};

const BRUTAL_LIGHT: &str = include_str!("../themes/brutalism-light.json");
const BRUTAL_DARK: &str = include_str!("../themes/brutalism-dark.json");

/// Install the brutalism renderer with a theme matching the system
/// appearance.
pub fn install(cx: &mut App) {
    install_with(cx, brutal_theme_for(cx.window_appearance()));
}

/// Install the brutalism renderer with the bundled light theme
/// (regardless of system appearance).
pub fn install_with_default_theme(cx: &mut App) {
    let theme = Theme::from_json(BRUTAL_LIGHT).expect("brutalism-light.json is valid");
    install_with(cx, theme);
}

/// Install the brutalism renderer with a custom theme.
pub fn install_with(cx: &mut App, theme: Theme) {
    install_theme(cx, theme);
    register_brutal_renderers(cx);
}

fn brutal_theme_for(appearance: WindowAppearance) -> Theme {
    let json = match appearance {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => BRUTAL_DARK,
        _ => BRUTAL_LIGHT,
    };
    Theme::from_json(json).expect("brutalism theme json is valid")
}

/// Register all 56 brutalist `XxxRenderer` impls against the core
/// `RendererRegistry`. Public so a caller who already installed
/// the theme (e.g. for tests) can still wire up the brutalist
/// look without re-installing the theme.
pub fn register_brutal_renderers(cx: &mut App) {
    // Actions (5)
    cx.register_renderer_arc::<m::Button, dyn ButtonRenderer>(Arc::new(BrutalButtonRenderer));
    cx.register_renderer_arc::<m::IconButton, dyn IconButtonRenderer>(Arc::new(
        BrutalIconButtonRenderer,
    ));
    cx.register_renderer_arc::<m::ToggleButton, dyn ToggleButtonRenderer>(Arc::new(
        BrutalToggleButtonRenderer,
    ));
    cx.register_renderer_arc::<m::SplitButton, dyn SplitButtonRenderer>(Arc::new(
        BrutalSplitButtonRenderer,
    ));
    cx.register_renderer_arc::<m::ButtonGroup, dyn ButtonGroupRenderer>(Arc::new(
        BrutalButtonGroupRenderer,
    ));

    // Display (14)
    cx.register_renderer_arc::<m::Label, dyn LabelRenderer>(Arc::new(BrutalLabelRenderer));
    cx.register_renderer_arc::<m::Heading, dyn HeadingRenderer>(Arc::new(BrutalHeadingRenderer));
    cx.register_renderer_arc::<m::Divider, dyn DividerRenderer>(Arc::new(BrutalDividerRenderer));
    cx.register_renderer_arc::<m::FocusRing, dyn FocusRingRenderer>(Arc::new(
        BrutalFocusRingRenderer,
    ));
    cx.register_renderer_arc::<m::Badge, dyn BadgeRenderer>(Arc::new(BrutalBadgeRenderer));
    cx.register_renderer_arc::<m::Tag, dyn TagRenderer>(Arc::new(BrutalTagRenderer));
    cx.register_renderer_arc::<m::Skeleton, dyn SkeletonRenderer>(Arc::new(BrutalSkeletonRenderer));
    cx.register_renderer_arc::<m::ProgressBar, dyn ProgressBarRenderer>(Arc::new(
        BrutalProgressBarRenderer,
    ));
    cx.register_renderer_arc::<m::EmptyState, dyn EmptyStateRenderer>(Arc::new(
        BrutalEmptyStateRenderer,
    ));
    cx.register_renderer_arc::<m::KeybindingDisplay, dyn KeybindingDisplayRenderer>(Arc::new(
        BrutalKeybindingDisplayRenderer,
    ));
    cx.register_renderer_arc::<m::ShortcutHint, dyn ShortcutHintRenderer>(Arc::new(
        BrutalShortcutHintRenderer,
    ));
    cx.register_renderer_arc::<m::Icon, dyn CoreIconRenderer>(Arc::new(BrutalIconRenderer));
    cx.register_renderer_arc::<m::Text, dyn TextRenderer>(Arc::new(BrutalTextRenderer));
    cx.register_renderer_arc::<m::Spacer, dyn SpacerRenderer>(Arc::new(BrutalSpacerRenderer));

    // Surfaces (5)
    cx.register_renderer_arc::<m::Tooltip, dyn TooltipRenderer>(Arc::new(BrutalTooltipRenderer));
    cx.register_renderer_arc::<m::Avatar, dyn AvatarRenderer>(Arc::new(BrutalAvatarRenderer));
    cx.register_renderer_arc::<m::Panel, dyn PanelRenderer>(Arc::new(BrutalPanelRenderer));
    cx.register_renderer_arc::<m::Card, dyn CardRenderer>(Arc::new(BrutalCardRenderer));
    cx.register_renderer_arc::<m::Image, dyn ImageRenderer>(Arc::new(BrutalImageRenderer));

    // Inputs (9)
    cx.register_renderer_arc::<m::TextInput, dyn TextInputRenderer>(Arc::new(
        BrutalTextInputRenderer,
    ));
    cx.register_renderer_arc::<m::TextArea, dyn TextAreaRenderer>(Arc::new(BrutalTextAreaRenderer));
    cx.register_renderer_arc::<m::PasswordInput, dyn PasswordInputRenderer>(Arc::new(
        BrutalPasswordInputRenderer,
    ));
    cx.register_renderer_arc::<m::NumberInput, dyn NumberInputRenderer>(Arc::new(
        BrutalNumberInputRenderer,
    ));
    cx.register_renderer_arc::<m::FilePathInput, dyn FilePathInputRenderer>(Arc::new(
        BrutalFilePathInputRenderer,
    ));
    cx.register_renderer_arc::<m::SearchInput, dyn SearchInputRenderer>(Arc::new(
        BrutalSearchInputRenderer,
    ));
    cx.register_renderer_arc::<m::Select, dyn SelectRenderer>(Arc::new(BrutalSelectRenderer));
    cx.register_renderer_arc::<m::ComboBox, dyn ComboBoxRenderer>(Arc::new(BrutalComboBoxRenderer));
    cx.register_renderer_arc::<m::KeybindingInput, dyn KeybindingInputRenderer>(Arc::new(
        BrutalKeybindingInputRenderer,
    ));

    // Controls (5)
    cx.register_renderer_arc::<m::Switch, dyn SwitchRenderer>(Arc::new(BrutalSwitchRenderer));
    cx.register_renderer_arc::<m::Checkbox, dyn CheckboxRenderer>(Arc::new(BrutalCheckboxRenderer));
    cx.register_renderer_arc::<m::Radio, dyn RadioRenderer>(Arc::new(BrutalRadioRenderer));
    cx.register_renderer_arc::<m::RadioGroup, dyn RadioGroupRenderer>(Arc::new(
        BrutalRadioGroupRenderer,
    ));
    cx.register_renderer_arc::<m::Slider, dyn SliderRenderer>(Arc::new(BrutalSliderRenderer));

    // Overlays (6)
    cx.register_renderer_arc::<m::Modal, dyn ModalRenderer>(Arc::new(BrutalModalRenderer));
    cx.register_renderer_arc::<m::Popover, dyn PopoverRenderer>(Arc::new(BrutalPopoverRenderer));
    cx.register_renderer_arc::<m::DropdownMenu, dyn DropdownMenuRenderer>(Arc::new(
        BrutalDropdownMenuRenderer,
    ));
    cx.register_renderer_arc::<m::Disclosure, dyn DisclosureRenderer>(Arc::new(
        BrutalDisclosureRenderer,
    ));
    cx.register_renderer_arc::<m::Overlay, dyn OverlayRenderer>(Arc::new(BrutalOverlayRenderer));
    cx.register_renderer_arc::<m::Menu, dyn MenuRenderer>(Arc::new(BrutalMenuRenderer));

    // Notifications (2)
    cx.register_renderer_arc::<m::Toast, dyn ToastRenderer>(Arc::new(BrutalToastRenderer));
    cx.register_renderer_arc::<m::Notification, dyn NotificationRenderer>(Arc::new(
        BrutalNotificationRenderer,
    ));

    // Lists (10)
    cx.register_renderer_arc::<m::ListItem, dyn ListItemRenderer>(Arc::new(BrutalListItemRenderer));
    cx.register_renderer_arc::<m::Listbox, dyn ListboxRenderer>(Arc::new(BrutalListboxRenderer));
    cx.register_renderer_arc::<m::GridView, dyn GridViewRenderer>(Arc::new(
        BrutalGridViewRenderer,
    ));
    cx.register_renderer_arc::<m::TreeItem, dyn TreeItemRenderer>(Arc::new(BrutalTreeItemRenderer));
    cx.register_renderer_arc::<m::Tree, dyn TreeRenderer>(Arc::new(BrutalTreeRenderer));
    cx.register_renderer_arc::<m::Form, dyn FormRenderer>(Arc::new(BrutalFormRenderer));
    cx.register_renderer_arc::<m::FormField, dyn FormFieldRenderer>(Arc::new(
        BrutalFormFieldRenderer,
    ));
    cx.register_renderer_arc::<m::Table, dyn TableRenderer>(Arc::new(BrutalTableRenderer));
    cx.register_renderer_arc::<m::VirtualList, dyn VirtualListRenderer>(Arc::new(
        BrutalVirtualListRenderer,
    ));
    cx.register_renderer_arc::<m::UniformVirtualList, dyn UniformVirtualListRenderer>(Arc::new(
        BrutalUniformVirtualListRenderer,
    ));
}
