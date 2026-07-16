//! Brutalist surface renderers: `Tooltip`, `Avatar`, `Panel`,
//! `Card`.

use gpui::{
    App, AppContext, Context, CursorStyle, Div, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, Render, StatefulInteractiveElement, Styled, Window, div, px,
};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::style::{
    BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, BRUTAL_RADIUS, BRUTAL_SMALL_BORDER_WIDTH,
    brutal_border_color,
};

// =====================================================================
// Tooltip
// =====================================================================

pub use yororen_ui_core::renderer::tooltip::{TooltipRenderState, TooltipRenderer};

pub struct BrutalTooltipRenderer;

/// View rendered by gpui's `hoverable_tooltip` builder for the
/// brutalism tooltip panel.
struct BrutalTooltipView {
    text: String,
    bg: Hsla,
    fg: Hsla,
    pad_top: Pixels,
    font_size: Pixels,
    border_radius: Pixels,
    max_width: Pixels,
}

impl Render for BrutalTooltipView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::div()
            .bg(self.bg)
            .text_color(self.fg)
            .p(self.pad_top)
            .text_size(self.font_size)
            .rounded(self.border_radius)
            .max_w(self.max_width)
            .child(self.text.clone())
    }
}

// Inherent helpers — *not* part of the trait surface.
impl BrutalTooltipRenderer {
    pub fn bg(&self, _: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn fg(&self, _: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn padding(&self, _: &TooltipRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.tooltip.padding_x")
            .unwrap_or(10.0) as f32;
        let v = theme
            .get_number("tokens.control.tooltip.padding_y")
            .unwrap_or(6.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    pub fn font_size(&self, _: &TooltipRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tooltip.font_size")
            .unwrap_or(12.0) as f32)
    }
    pub fn border_radius(&self, _: &TooltipRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl TooltipRenderer for BrutalTooltipRenderer {
    fn compose(
        &self,
        props: &mut yororen_ui_core::headless::tooltip::TooltipProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TooltipRenderState {
            has_custom_bg: props.has_custom_bg,
            has_custom_fg: props.has_custom_fg,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let fs = self.font_size(&state, theme);
        let r = self.border_radius(&state, theme);
        let max_w = px(theme
            .get_number("tokens.control.tooltip.max_width")
            .unwrap_or(240.0) as f32);

        // The trigger is wrapped in a `Stateful<Div>` so we can attach
        // gpui's `hoverable_tooltip`. The floating panel is created by
        // gpui on hover and styled with brutalism tokens.
        let mut outer = gpui::div().flex().flex_col().items_start();

        if let Some(t) = props.trigger.take() {
            let text = props.text.clone();
            let trigger_id = format!("{}-trigger", props.id);
            outer = outer.child(gpui::div().id(trigger_id).child(t).hoverable_tooltip(
                move |_window, cx| {
                    cx.new(|_cx| BrutalTooltipView {
                        text: text.clone(),
                        bg,
                        fg,
                        pad_top: pad.top,
                        font_size: fs,
                        border_radius: r,
                        max_width: max_w,
                    })
                    .into()
                },
            ));
        }

        outer
    }
}

// =====================================================================
// Avatar
// =====================================================================

pub use yororen_ui_core::renderer::avatar::{AvatarRenderState, AvatarRenderer};

pub struct BrutalAvatarRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalAvatarRenderer {
    pub fn default_bg(&self, _: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }

    pub fn border_radius(&self, _: &AvatarRenderState, _: &Theme) -> Pixels {
        // Brutalism: square avatars (no pill, no radius).
        px(BRUTAL_RADIUS)
    }

    pub fn status_dot_size(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.avatar.status_dot_size")
            .unwrap_or(12.0) as f32)
    }

    pub fn status_inset(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.avatar.status_inset")
            .unwrap_or(2.0) as f32)
    }

    pub fn status_border_w(&self, _: &AvatarRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.avatar.border_w")
            .unwrap_or(BRUTAL_SMALL_BORDER_WIDTH as f64) as f32)
    }

    pub fn status_border_color(&self, _: &AvatarRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    /// Initials / label colour. Reads `content.primary` from
    /// the theme so the text contrasts with the surface
    /// background in both light and dark modes. Without this
    /// helper the `div().child(text)` would inherit gpui's
    /// default (`#000000`) which is invisible against the dark
    /// avatar background.
    pub fn label_color(&self, _: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
}

impl AvatarRenderer for BrutalAvatarRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::avatar::AvatarProps, cx: &App) -> Div {
        let theme = cx.theme();
        let state = AvatarRenderState {
            has_custom_bg: props.has_custom_bg,
            has_status: props.has_status,
            is_circle: props.circle,
        };
        let bg = self.default_bg(&state, theme);
        let r = self.border_radius(&state, theme);
        let label_color = self.label_color(&state, theme);
        let size = props.size.unwrap_or(px(40.0));
        // Initials font sized at ~40% of avatar height so 2-letter
        // initials always fit inside the box.
        let font_size = size * 0.4;
        let label_text: Option<String> = if let Some(initials) = &props.initials {
            Some(initials.clone())
        } else {
            props.name.as_ref().map(|n| initials_from_name(n.as_ref()))
        };
        let content = if let Some(src) = &props.src {
            // `src` wins over initials. The corner radius goes on
            // the `img` itself — gpui paints images with their own
            // style's corner radii, while a parent's
            // `overflow_hidden` clips to a plain rectangle.
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
                    .bg(theme
                        .get_color("status.success.bg")
                        .unwrap_or(BRUTAL_BORDER)),
            );
        }
        el
    }
}

/// Extract up to 2 uppercase initials from a person's name. For
/// Latin / Cyrillic / Greek alphabets, takes the first letter of
/// the first and last whitespace-separated tokens (`"Jane Doe"` →
/// `"JD"`). For CJK names returns only the first character
/// (`"张三"` → `"张"`).
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

fn is_cjk_char(c: char) -> bool {
    matches!(
        c as u32,
        0x3040..=0x309F
        | 0x30A0..=0x30FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xAC00..=0xD7AF
        | 0xF900..=0xFAFF
    )
}

// =====================================================================
// Panel
// =====================================================================

pub use yororen_ui_core::renderer::panel::{PanelRenderState, PanelRenderer};

pub struct BrutalPanelRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalPanelRenderer {
    pub fn bg(&self, _: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }

    pub fn border(&self, _: &PanelRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    pub fn padding(&self, _: &PanelRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.panel.padding")
            .unwrap_or(16.0) as f32;
        Edges::all(px(p))
    }

    pub fn border_radius(&self, _: &PanelRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn shadow_alpha(&self, _: &PanelRenderState, _: &Theme) -> f32 {
        1.0
    }

    pub fn title_color(&self, _: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
}

impl PanelRenderer for BrutalPanelRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::panel::PanelProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = PanelRenderState {
            has_custom_bg: false,
            has_custom_border: false,
            has_custom_padding: false,
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let pad = self.padding(&state, theme);
        let r = self.border_radius(&state, theme);
        let title_fg = self.title_color(&state, theme);
        let mut el = div()
            .flex()
            .flex_col()
            .bg(bg)
            .border(px(BRUTAL_BORDER_WIDTH))
            .border_color(border)
            .p(pad.top)
            .rounded(r);
        if let Some(title) = &props.title {
            el = el.child(div().text_color(title_fg).pb(px(6.)).child(title.clone()));
        }
        el
    }
}

// =====================================================================
// Card
// =====================================================================

pub use yororen_ui_core::renderer::card::{CardRenderState, CardRenderer};

pub struct BrutalCardRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalCardRenderer {
    pub fn bg(&self, _: &CardRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }

    pub fn border(&self, _: &CardRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    pub fn padding(&self, _: &CardRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.card.padding")
            .unwrap_or(16.0) as f32;
        Edges::all(px(p))
    }

    pub fn border_radius(&self, _: &CardRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn shadow_alpha(&self, _: &CardRenderState, _: &Theme) -> f32 {
        1.0
    }
}

impl CardRenderer for BrutalCardRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::card::CardProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = CardRenderState {
            has_custom_bg: false,
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let pad = self.padding(&state, theme);
        let r = self.border_radius(&state, theme);
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .bg(bg)
            .border(px(BRUTAL_BORDER_WIDTH))
            .border_color(border)
            .p(pad.top)
            .rounded(r)
            .cursor(if props.interactive {
                CursorStyle::PointingHand
            } else {
                CursorStyle::Arrow
            })
    }
}

// End of card impl.

// =====================================================================
// Image
// =====================================================================

pub use yororen_ui_core::renderer::image::{ImageRenderState, ImageRenderer};

use gpui::Stateful;
use yororen_ui_core::headless::image::ImageProps;

pub struct BrutalImageRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalImageRenderer {
    pub fn border(&self, _state: &ImageRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
}

impl ImageRenderer for BrutalImageRenderer {
    fn compose(&self, props: &ImageProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = ImageRenderState {};
        let bd = self.border(&state, theme);
        let img = gpui::img(props.source.as_gpui_source());
        gpui::div()
            .id(props.id.clone())
            .border(px(BRUTAL_BORDER_WIDTH))
            .border_color(bd)
            .rounded(px(BRUTAL_RADIUS))
            .overflow_hidden()
            .child(img.size_full())
    }
}

// End of image impl.
