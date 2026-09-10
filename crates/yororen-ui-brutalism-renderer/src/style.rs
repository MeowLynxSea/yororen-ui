//! Shared constants and helpers for the brutalism renderers.
//!
//! Every `XxxRenderer` implementation in this crate reads from here
//! for the recurring brutalism values (border color, border width,
//! radius, font family, offset shadow) so the 55 renderers stay in
//! stylistic lockstep.

use gpui::{Hsla, px};
use yororen_ui_core::renderer::spec::ShadowSpec;
use yororen_ui_core::theme::Theme;

pub const BRUTAL_BORDER: Hsla = gpui::hsla(0.0, 0.0, 0.0, 1.0);
pub const BRUTAL_BORDER_WHITE: Hsla = gpui::hsla(0.0, 0.0, 1.0, 1.0);

pub const BRUTAL_BORDER_WIDTH: f32 = 3.0;
pub const BRUTAL_SMALL_BORDER_WIDTH: f32 = 2.0;
pub const BRUTAL_LARGE_BORDER_WIDTH: f32 = 4.0;

pub const BRUTAL_RADIUS: f32 = 0.0;

pub const BRUTAL_FONT_FAMILY: &str = "ui-monospace, 'Courier New', monospace";

pub const BRUTAL_DISABLED_OPACITY: f32 = 0.6;

/// Border color as defined by the theme (`border.default`).
/// In the light theme this is pure black; in the dark theme,
/// pure white.
pub fn brutal_border_color(theme: &Theme) -> Hsla {
    theme.get_color("border.default").unwrap_or(BRUTAL_BORDER)
}

/// Hard vertical offset shadow — the defining neo-brutalism touch.
/// Reads Y offset from `shadow.default.offset_y`; blur is always 0.
/// (The current `ShadowSpec` only supports a single Y offset, not
/// both X and Y, so the shadow is downward-only.)
pub fn brutal_shadow(theme: &Theme) -> ShadowSpec {
    let offset_y = theme.get_number("shadow.default.offset_y").unwrap_or(4.0) as f32;
    let color = theme
        .get_color("shadow.default.color")
        .unwrap_or(BRUTAL_BORDER);
    ShadowSpec {
        blur: px(0.0),
        offset_y: px(offset_y),
        color,
    }
}

/// Larger Y offset shadow for raised surfaces (cards, panels).
pub fn brutal_shadow_raised(theme: &Theme) -> ShadowSpec {
    let offset_y = theme.get_number("shadow.raised.offset_y").unwrap_or(6.0) as f32;
    let color = theme
        .get_color("shadow.raised.color")
        .unwrap_or(BRUTAL_BORDER);
    ShadowSpec {
        blur: px(0.0),
        offset_y: px(offset_y),
        color,
    }
}

/// Largest Y offset shadow for overlays (modals, popovers).
pub fn brutal_shadow_overlay(theme: &Theme) -> ShadowSpec {
    let offset_y = theme.get_number("shadow.overlay.offset_y").unwrap_or(8.0) as f32;
    let color = theme
        .get_color("shadow.overlay.color")
        .unwrap_or(BRUTAL_BORDER);
    ShadowSpec {
        blur: px(0.0),
        offset_y: px(offset_y),
        color,
    }
}
