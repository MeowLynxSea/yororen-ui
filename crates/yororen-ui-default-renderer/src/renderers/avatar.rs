//! `TokenAvatarRenderer` — default `AvatarRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::avatar::AvatarProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::avatar::{AvatarRenderState, AvatarRenderer};

pub struct TokenAvatarRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenAvatarRenderer {
    pub fn default_bg(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }

    pub fn border_radius(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels {
        if state.is_circle {
            gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
        } else {
            gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
        }
    }

    pub fn status_dot_size(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.status_dot_size")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn status_inset(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.status_inset")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn status_border_w(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.avatar.border_w")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn status_border_color(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
}

impl AvatarRenderer for TokenAvatarRenderer {
    fn compose(&self, props: &AvatarProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = AvatarRenderState {
            has_custom_bg: props.has_custom_bg,
            has_status: props.has_status,
            is_circle: props.circle,
        };
        let bg = self.default_bg(&state, theme);
        let r = self.border_radius(&state, theme);
        let size = props.size.unwrap_or(gpui::px(40.0));
        // Initials font sized at ~40% of avatar height so 2-letter
        // initials always fit inside the circle/square.
        let font_size = size * 0.4;
        let label_text: Option<String> = if let Some(initials) = &props.initials {
            Some(initials.clone())
        } else {
            props.name.as_ref().map(|n| initials_from_name(n.as_ref()))
        };
        let label_color: Hsla = theme.get_color("content.primary").unwrap_or_default();
        // `src` wins over initials: while the image loads the `bg`
        // placeholder shows through, then the image covers the box.
        // The corner radius goes on the `img` itself — gpui paints
        // images with their own style's corner radii, while a
        // parent's `overflow_hidden` clips to a plain rectangle.
        let content = if let Some(src) = &props.src {
            div().size_full().child(
                gpui::img(yororen_ui_core::headless::image::resolve_resource(
                    src.as_ref(),
                ))
                .size_full()
                .rounded(r),
            )
        } else if let Some(text) = label_text {
            div()
                .text_size(font_size)
                .text_color(label_color)
                .child(text)
        } else {
            div()
        };
        let mut el = div()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .rounded(r)
            .size(size)
            .child(content);
        if props.has_status {
            let dot = self.status_dot_size(&state, theme);
            let inset = self.status_inset(&state, theme);
            let bw = self.status_border_w(&state, theme);
            let bc = self.status_border_color(&state, theme);
            el = el.child(
                div()
                    .absolute()
                    .right(inset)
                    .bottom(inset)
                    .size(dot)
                    .rounded(dot / 2.)
                    .border(bw)
                    .border_color(bc)
                    .bg(theme.get_color("status.success.bg").unwrap_or_default()),
            );
        }
        el
    }
}

/// Extract up to 2 uppercase initials from a person's name. For
/// Latin / Cyrillic / Greek alphabets, takes the first letter of
/// the first and last whitespace-separated tokens (`"Jane Doe"` →
/// `"JD"`, `"Cher"` → `"C"`). For CJK names (Chinese / Japanese /
/// Korean) it returns only the **first character** (`"张三"` →
/// `"张"`, `"山田太郎"` → `"山"`), since each glyph is already a
/// full unit and stacking two would crowd the avatar.
fn initials_from_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some(first) = trimmed.chars().next()
        && is_cjk_char(first)
    {
        return first.to_string();
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    if let Some(first) = parts.first().and_then(|w| w.chars().next()) {
        for c in first.to_uppercase() {
            out.push(c);
        }
    }
    if parts.len() > 1
        && let Some(last) = parts.last().and_then(|w| w.chars().next())
    {
        for c in last.to_uppercase() {
            out.push(c);
        }
    }
    out
}

/// True if `c` is in a CJK script range (Chinese Hanzi, Japanese
/// Kanji / Hiragana / Katakana, Korean Hangul). Avatars treat
/// these specially because each glyph is a full word-unit, not a
/// letter, so the "first + last initial" heuristic doesn't apply.
fn is_cjk_char(c: char) -> bool {
    matches!(
        c as u32,
        0x3040..=0x309F   // Hiragana
        | 0x30A0..=0x30FF // Katakana
        | 0x3400..=0x4DBF // CJK Unified Ideographs Extension A
        | 0x4E00..=0x9FFF // CJK Unified Ideographs
        | 0xAC00..=0xD7AF // Hangul Syllables
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
    )
}

pub fn arc_avatar<T: AvatarRenderer + 'static>(r: T) -> Arc<dyn AvatarRenderer> {
    Arc::new(r)
}
