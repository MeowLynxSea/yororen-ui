//! Brutalist display renderers: `Label`, `Heading`, `Divider`,
//! `FocusRing`, `Badge`, `Tag`, `Skeleton`, `ProgressBar`,
//! `EmptyState`.

use gpui::{
    AbsoluteLength, AlignSelf, App, BorderStyle, Bounds, BoxShadow, Corners, CursorStyle,
    DefiniteLength, Div, Edges, Element, ElementId, FontWeight, GlobalElementId, Hsla,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, Length, Pixels, Position,
    SharedString, Stateful, StatefulInteractiveElement, Style, Styled, Window, hsla, point, px,
};
use std::sync::OnceLock;
use std::time::Instant;

use yororen_ui_core::animation::ease_in_out;
use yororen_ui_core::headless::badge::BadgeVariant;
use yororen_ui_core::headless::icon::IconSource;
use yororen_ui_core::headless::label::LabelProps;
use yororen_ui_core::renderer::spec::Edges as SpecEdges;
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;

use gpui::ParentElement;

use crate::style::{
    BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, BRUTAL_FONT_FAMILY, BRUTAL_RADIUS, brutal_border_color,
};

// =====================================================================
// Label
// =====================================================================

pub use yororen_ui_core::renderer::label::{LabelRenderState, LabelRenderer};

pub struct BrutalLabelRenderer;

// Inherent helpers — *not* part of the `LabelRenderer` trait
// surface.
impl BrutalLabelRenderer {
    pub fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla {
        if state.muted {
            theme
                .get_color("content.secondary")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }

    pub fn strong_weight(&self, _: &LabelRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.control.label.weight")
                .or_else(|| theme.get_number("tokens.typography.weight_medium"))
                .unwrap_or(700.0) as f32,
        )
    }

    pub fn family_mono(&self, _: &LabelRenderState, theme: &Theme) -> SharedString {
        theme
            .get_string("tokens.typography.family_mono")
            .unwrap_or(BRUTAL_FONT_FAMILY)
            .to_string()
            .into()
    }
}

impl LabelRenderer for BrutalLabelRenderer {
    fn compose(&self, props: &LabelProps, cx: &App) -> Div {
        let theme = cx.theme();
        let state = LabelRenderState {
            muted: props.muted,
            strong: props.strong,
            mono: props.mono,
            inherit_color: props.inherit_color,
            ellipsis: props.ellipsis,
            wrap: props.wrap,
            max_lines: props.max_lines,
        };
        let color = self.color(&state, theme);
        let weight = self.strong_weight(&state, theme);
        let family = self.family_mono(&state, theme);
        let mut el: Div = gpui::div();
        if !props.inherit_color {
            el = el.text_color(color);
        }
        if props.strong {
            el = el.font_weight(weight);
        }
        if props.mono {
            el = el.font_family(family);
        }
        if props.ellipsis {
            el = el.overflow_hidden().text_ellipsis().whitespace_nowrap();
        }
        if props.wrap {
            el = el.whitespace_normal();
        }
        if let Some(n) = props.max_lines {
            el = el.line_clamp(n).overflow_hidden();
        }
        el.child(props.text.clone())
    }
}

// =====================================================================
// Heading
// =====================================================================

pub use yororen_ui_core::renderer::heading::{HeadingRenderState, HeadingRenderer};

pub struct BrutalHeadingRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalHeadingRenderer {
    pub fn size(&self, state: &HeadingRenderState, theme: &Theme) -> Pixels {
        let path = match state.level {
            yororen_ui_core::headless::heading::HeadingLevel::H1 => {
                "tokens.control.heading.font_size_lg"
            }
            yororen_ui_core::headless::heading::HeadingLevel::H2 => {
                "tokens.control.heading.font_size_md"
            }
            _ => "tokens.control.heading.font_size_sm",
        };
        px(theme.get_number(path).unwrap_or(24.0) as f32)
    }

    pub fn weight(&self, state: &HeadingRenderState, theme: &Theme) -> FontWeight {
        let default = match state.level {
            yororen_ui_core::headless::heading::HeadingLevel::H1 => 800.0,
            _ => 800.0,
        };
        FontWeight(
            theme
                .get_number("tokens.control.heading.weight")
                .or_else(|| theme.get_number("tokens.typography.weight_bold"))
                .unwrap_or(default) as f32,
        )
    }

    pub fn color(&self, _: &HeadingRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
}

impl HeadingRenderer for BrutalHeadingRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::heading::HeadingProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = HeadingRenderState { level: props.level };
        let size = self.size(&state, theme);
        let weight = self.weight(&state, theme);
        let color = self.color(&state, theme);
        gpui::div()
            .text_color(color)
            .text_size(size)
            .font_weight(weight)
            .child(props.text.clone())
    }
}

// =====================================================================
// Divider
// =====================================================================

pub use yororen_ui_core::renderer::divider::{DividerRenderState, DividerRenderer};

pub struct BrutalDividerRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalDividerRenderer {
    pub fn color(&self, _: &DividerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.divider").unwrap_or(BRUTAL_BORDER)
    }
    pub fn thickness(&self, _: &DividerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.divider.thickness")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32)
    }
}

impl DividerRenderer for BrutalDividerRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::divider::DividerProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DividerRenderState {
            horizontal: props.horizontal,
        };
        let color = self.color(&state, theme);
        let thickness = self.thickness(&state, theme);
        let mut el = gpui::div().bg(color).flex_shrink_0();
        if props.horizontal {
            el.style().align_self = Some(AlignSelf::Stretch);
            el = el.w_full().h(thickness).min_h(thickness);
        } else {
            el.style().align_self = Some(AlignSelf::Stretch);
            el = el.h_full().w(thickness).min_w(thickness);
        }
        el
    }
}

// =====================================================================
// FocusRing
// =====================================================================

pub use yororen_ui_core::renderer::focus_ring::{FocusRingRenderState, FocusRingRenderer};

pub struct BrutalFocusRingRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalFocusRingRenderer {
    pub fn color(&self, _: &FocusRingRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }

    pub fn width(&self, _: &FocusRingRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.focus_ring.width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32)
    }

    /// Brutalism keeps everything sharp-cornered, so the focus
    /// ring is rendered with `BRUTAL_RADIUS` (0). The wrapper's
    /// `rounded(...)` is what the `box-shadow` follows.
    pub fn border_radius(&self, _: &FocusRingRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl FocusRingRenderer for BrutalFocusRingRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::focus_ring::FocusRingProps,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = FocusRingRenderState {
            has_custom_color: props.has_custom_color,
        };
        let color = self.color(&state, theme);
        let width = self.width(&state, theme);
        let radius = self.border_radius(&state, theme);
        gpui::div()
            .id(props.id.clone())
            .track_focus(&props.focus_handle)
            .rounded(radius)
            .shadow(vec![BoxShadow {
                color,
                offset: point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: width,
                inset: false,
            }])
    }
}

// =====================================================================
// Badge
// =====================================================================

pub use yororen_ui_core::renderer::badge::{BadgeRenderState, BadgeRenderer};

pub struct BrutalBadgeRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalBadgeRenderer {
    pub fn bg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        let key = match state.variant {
            BadgeVariant::Neutral => "neutral",
            BadgeVariant::Success => "success",
            BadgeVariant::Warning => "warning",
            BadgeVariant::Danger => "danger",
            BadgeVariant::Info => "info",
        };
        theme
            .get_color(&format!("status.{key}.bg"))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        let key = match state.variant {
            BadgeVariant::Neutral => "neutral",
            BadgeVariant::Success => "success",
            BadgeVariant::Warning => "warning",
            BadgeVariant::Danger => "danger",
            BadgeVariant::Info => "info",
        };
        theme
            .get_color(&format!("status.{key}.fg"))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn padding_x(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.badge.horizontal_padding")
            .unwrap_or(8.0) as f32)
    }

    pub fn height(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.badge.min_height")
            .unwrap_or(22.0) as f32)
    }

    pub fn font_size(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.badge.font_size")
            .unwrap_or(11.0) as f32)
    }

    pub fn font_weight(&self, _: &BadgeRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.control.badge.weight")
                .or_else(|| theme.get_number("tokens.typography.weight_bold"))
                .unwrap_or(800.0) as f32,
        )
    }

    pub fn border_radius(&self, _: &BadgeRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl BadgeRenderer for BrutalBadgeRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::badge::BadgeProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = BadgeRenderState {
            variant: props.variant,
            has_custom_tone: false,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let px_v = self.padding_x(&state, theme);
        let h = self.height(&state, theme);
        let fs = self.font_size(&state, theme);
        let fw = self.font_weight(&state, theme);
        let r = self.border_radius(&state, theme);
        gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .text_color(fg)
            .px(px_v)
            .h(h)
            .text_size(fs)
            .font_weight(fw)
            .rounded(r)
            .child(props.text.clone())
    }
}

// =====================================================================
// Tag
// =====================================================================

pub use yororen_ui_core::renderer::tag::{TagRenderState, TagRenderer};

pub struct BrutalTagRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalTagRenderer {
    pub fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme
                .get_color("action.primary.bg")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.has_custom_tone {
            theme
                .get_color("content.on_status")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    pub fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.has_custom_tone {
            theme
                .get_color("content.on_status")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.fg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    pub fn min_height(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tag.min_height")
            .unwrap_or(28.0) as f32)
    }

    pub fn padding_x(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tag.horizontal_padding")
            .unwrap_or(12.0) as f32)
    }

    pub fn font_size(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.typography.font_size_xs")
            .unwrap_or(12.0) as f32)
    }

    pub fn font_weight(&self, _: &TagRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_bold")
                .unwrap_or(700.0) as f32,
        )
    }

    pub fn border_radius(&self, _: &TagRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn close_size(&self, _: &TagRenderState, _: &Theme) -> Pixels {
        px(16.0)
    }

    pub fn close_hover_bg(&self, _: &TagRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
}

impl TagRenderer for BrutalTagRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::tag::TagProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TagRenderState {
            selected: props.selected,
            has_custom_tone: false,
            closable: props.closable,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let h = self.min_height(&state, theme);
        let p = self.padding_x(&state, theme);
        let fs = self.font_size(&state, theme);
        let fw = self.font_weight(&state, theme);
        let r = self.border_radius(&state, theme);
        let mut el = gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .min_h(h)
            .px(p)
            .text_size(fs)
            .font_weight(fw)
            .rounded(r)
            .gap(p / 2.)
            .border_2()
            .border_color(brutal_border_color(theme))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else if props.on_click.is_some() {
                CursorStyle::PointingHand
            } else {
                CursorStyle::Arrow
            })
            .child(props.label.clone());
        if props.closable {
            let close_size = self.close_size(&state, theme);
            // `on_click` lives on `StatefulInteractiveElement`,
            // which requires an id. Derive a stable, unique id
            // from the tag's own id so the close button gets a
            // distinct identity.
            let close_id: gpui::ElementId = match &props.id {
                gpui::ElementId::Name(name) => {
                    let mut s = name.to_string();
                    s.push_str("__close");
                    s.into()
                }
                _ => "brutal_tag_close".into(),
            };
            let mut close_btn = gpui::div()
                .id(close_id)
                .flex()
                .items_center()
                .justify_center()
                .size(close_size)
                .rounded(close_size / 2.)
                .cursor(gpui::CursorStyle::PointingHand)
                .child("×");
            if !props.disabled
                && let Some(f) = props.on_close.clone()
            {
                close_btn = close_btn.on_click(move |ev, window, cx: &mut gpui::App| {
                    cx.stop_propagation();
                    f(ev, window, cx);
                });
            }
            el = el.child(close_btn);
        }
        el
    }
}

// =====================================================================
// Skeleton
// =====================================================================

pub use yororen_ui_core::renderer::skeleton::{SkeletonRenderState, SkeletonRenderer};

pub struct BrutalSkeletonRenderer;

impl BrutalSkeletonRenderer {
    pub fn bg(&self, _: &SkeletonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block {
            px(theme
                .get_number("tokens.control.skeleton.block_min_h")
                .unwrap_or(48.0) as f32)
        } else {
            px(theme
                .get_number("tokens.control.skeleton.line_h")
                .unwrap_or(16.0) as f32)
        }
    }
    pub fn border_radius(&self, _: &SkeletonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

/// Pulse opacity range (matches `yororen_ui_core::animation::preset::defaults`).
const BRUTAL_SKELETON_PULSE_MIN: f32 = 0.55;
const BRUTAL_SKELETON_PULSE_MAX: f32 = 0.95;

/// Animation epoch — all skeletons in the app pulse in sync,
/// which is the standard loading-animation behavior. Captured
/// once on first paint via `OnceLock`.
static BRUTAL_SKELETON_PULSE_EPOCH: OnceLock<Instant> = OnceLock::new();

/// A `Length` of zero pixels for `Edges::all` — pins the
/// absolutely-positioned overlay to all four sides of its parent
/// Div (the "fill the parent" idiom).
const BRUTAL_ZERO_LENGTH: Length = Length::Definite(DefiniteLength::Absolute(
    AbsoluteLength::Pixels(gpui::px(0.)),
));

/// Custom `gpui::Element` that paints a single rounded quad with
/// a time-varying alpha, producing the skeleton pulse animation.
struct BrutalSkeletonPulseElement {
    bg: Hsla,
    radius: Pixels,
    duration_ms: u64,
}

impl IntoElement for BrutalSkeletonPulseElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for BrutalSkeletonPulseElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            position: Position::Absolute,
            inset: Edges::all(BRUTAL_ZERO_LENGTH),
            ..Default::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let _ = BRUTAL_SKELETON_PULSE_EPOCH.get_or_init(Instant::now);
        window.request_animation_frame();
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let epoch = BRUTAL_SKELETON_PULSE_EPOCH.get_or_init(Instant::now);
        let elapsed_ms = epoch.elapsed().as_millis() as u64;
        let progress = if self.duration_ms == 0 {
            0.0
        } else {
            (elapsed_ms % self.duration_ms) as f32 / self.duration_ms as f32
        };
        // Triangle wave 0 → 1 → 0 so the alpha ramps UP for
        // the first half, then BACK DOWN — a true "breath"
        // instead of a sawtooth that snaps from MAX back to MIN
        // at the cycle boundary.
        let tri = if progress < 0.5 {
            progress * 2.0
        } else {
            2.0 - progress * 2.0
        };
        let eased = ease_in_out(tri);
        let alpha_mult = BRUTAL_SKELETON_PULSE_MIN
            + (BRUTAL_SKELETON_PULSE_MAX - BRUTAL_SKELETON_PULSE_MIN) * eased;
        let color = hsla(self.bg.color.hue.into_degrees() / 360.0, self.bg.saturation, self.bg.lightness, self.bg.alpha * alpha_mult);

        window.paint_quad(gpui::PaintQuad {
            bounds,
            corner_radii: Corners::all(self.radius).clamp_radii_for_quad_size(bounds.size),
            background: color.into(),
            border_color: hsla(0., 0., 0., 0.),
            border_widths: Edges::default(),
            border_style: BorderStyle::default(),
        });
    }
}

impl SkeletonRenderer for BrutalSkeletonRenderer {
    fn compose(&self, props: &yororen_ui_core::headless::skeleton::SkeletonProps, cx: &App) -> Div {
        let theme = cx.theme();
        let state = SkeletonRenderState {
            block: props.block,
            block_sharp: props.block_sharp,
        };
        let bg = self.bg(&state, theme);
        let min_h = self.min_height(&state, theme);
        let radius = self.border_radius(&state, theme);
        let duration_ms = theme
            .get_number("motion.duration_skeleton_pulse")
            .unwrap_or(1100.0) as u64;

        let mut el = gpui::div().rounded(radius);
        if let Some(w) = props.w {
            el = el.w(w);
        }
        if let Some(h) = props.h {
            el = el.h(h);
        } else {
            el = el.min_h(min_h);
        }
        el.child(BrutalSkeletonPulseElement {
            bg,
            radius,
            duration_ms,
        })
    }
}

// =====================================================================
// ProgressBar
// =====================================================================

pub use yororen_ui_core::renderer::progress::{ProgressBarRenderState, ProgressBarRenderer};

pub struct BrutalProgressBarRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalProgressBarRenderer {
    pub fn track(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }

    pub fn fill(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn height(&self, _: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.progress.height")
            .unwrap_or(28.0) as f32)
    }

    pub fn border_color(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    pub fn border_radius(&self, _: &ProgressBarRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl ProgressBarRenderer for BrutalProgressBarRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::progress::ProgressBarProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ProgressBarRenderState {
            indeterminate: props.indeterminate,
            has_custom_height: props.has_custom_height,
        };
        let track = self.track(&state, theme);
        let fill = self.fill(&state, theme);
        let h = self.height(&state, theme);
        let bc = self.border_color(&state, theme);
        let r = self.border_radius(&state, theme);
        let ratio = if props.indeterminate || props.max <= 0.0 {
            0.0
        } else {
            (props.value / props.max).clamp(0.0, 1.0)
        };
        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h(h)
            .bg(track)
            .rounded(r)
            .border_2()
            .border_color(bc)
            .child(
                gpui::div()
                    .h_full()
                    .w(gpui::relative(ratio))
                    .bg(fill)
                    .rounded(r),
            )
    }
}

// =====================================================================
// EmptyState
// =====================================================================

pub use yororen_ui_core::renderer::empty_state::{EmptyStateRenderState, EmptyStateRenderer};

pub struct BrutalEmptyStateRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalEmptyStateRenderer {
    pub fn icon_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn title_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn body_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.secondary")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn padding(&self, _: &EmptyStateRenderState, theme: &Theme) -> SpecEdges<Pixels> {
        let p = theme
            .get_number("tokens.control.empty_state.padding")
            .unwrap_or(32.0) as f32;
        SpecEdges::all(px(p))
    }
    pub fn icon_size(&self, _: &EmptyStateRenderState, _: &Theme) -> Pixels {
        px(48.0)
    }
    pub fn gap(&self, _: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.loose").unwrap_or(16.0) as f32)
    }
}

impl EmptyStateRenderer for BrutalEmptyStateRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::empty_state::EmptyStateProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = EmptyStateRenderState {};
        let ic = self.icon_color(&state, theme);
        let tc = self.title_color(&state, theme);
        let bc = self.body_color(&state, theme);
        let pad = self.padding(&state, theme);
        let is = self.icon_size(&state, theme);
        let g = self.gap(&state, theme);
        let mut el = gpui::div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p(pad.top)
            .gap(g);
        if let Some(icon) = &props.icon {
            // Resolve the icon source the same way `IconProps::render`
            // does: builtin names map to `icons/<name>.svg`; resource
            // paths pass through.
            let path: SharedString = match icon {
                IconSource::Builtin(name) => format!("icons/{name}.svg").into(),
                IconSource::Resource(p) => p.clone(),
            };
            el = el.child(
                gpui::svg()
                    .path(path)
                    .size(is)
                    .text_color(ic)
                    .into_any_element(),
            );
        }
        if let Some(title) = &props.title {
            el = el.child(gpui::div().text_color(tc).child(title.clone()));
        }
        if let Some(desc) = &props.description {
            el = el.child(gpui::div().text_color(bc).child(desc.clone()));
        }
        el
    }
}

// End of empty-state impl.

// =====================================================================
// KeybindingDisplay
// =====================================================================

pub use yororen_ui_core::renderer::keybinding_display::{
    KeybindingDisplayRenderState, KeybindingDisplayRenderer,
};

pub struct BrutalKeybindingDisplayRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalKeybindingDisplayRenderer {
    pub fn kbd_bg(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn kbd_fg(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn padding_x(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.kbd_padding_x")
            .unwrap_or(6.0) as f32)
    }
    pub fn padding_y(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.kbd_padding_y")
            .unwrap_or(2.0) as f32)
    }
    pub fn gap(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.separator_gap")
            .unwrap_or(4.0) as f32)
    }
    pub fn font_family(
        &self,
        _state: &KeybindingDisplayRenderState,
        theme: &Theme,
    ) -> SharedString {
        theme
            .get_string("tokens.typography.family_mono")
            .unwrap_or(BRUTAL_FONT_FAMILY)
            .to_string()
            .into()
    }
}

impl KeybindingDisplayRenderer for BrutalKeybindingDisplayRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::keybinding_display::KeybindingDisplayProps,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = KeybindingDisplayRenderState {};
        let kbd_bg = self.kbd_bg(&state, theme);
        let kbd_fg = self.kbd_fg(&state, theme);
        let bd = self.border(&state, theme);
        let px_h = self.padding_x(&state, theme);
        let px_v = self.padding_y(&state, theme);
        let g = self.gap(&state, theme);
        let family = self.font_family(&state, theme);
        let mut row = gpui::div()
            .id(props.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(g);
        for key in &props.keys {
            row = row.child(
                gpui::div()
                    .bg(kbd_bg)
                    .text_color(kbd_fg)
                    .border(px(BRUTAL_BORDER_WIDTH))
                    .border_color(bd)
                    .rounded(px(BRUTAL_RADIUS))
                    .px(px_h)
                    .py(px_v)
                    .font_family(family.clone())
                    .text_size(px(12.))
                    .child(key.clone()),
            );
        }
        row
    }
}

// End of keybinding-display impl.

// =====================================================================
// ShortcutHint
// =====================================================================

pub use yororen_ui_core::renderer::shortcut_hint::{ShortcutHintRenderState, ShortcutHintRenderer};

pub struct BrutalShortcutHintRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalShortcutHintRenderer {
    pub fn label_fg(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.secondary")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn kbd_bg(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn kbd_fg(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn padding_x(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.kbd_padding_x")
            .unwrap_or(6.0) as f32)
    }
    pub fn padding_y(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.kbd_padding_y")
            .unwrap_or(2.0) as f32)
    }
    pub fn key_gap(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.separator_gap")
            .unwrap_or(4.0) as f32)
    }
    pub fn label_gap(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0) as f32)
    }
    pub fn font_family(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> SharedString {
        theme
            .get_string("tokens.typography.family_mono")
            .unwrap_or(BRUTAL_FONT_FAMILY)
            .to_string()
            .into()
    }
}

impl ShortcutHintRenderer for BrutalShortcutHintRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::shortcut_hint::ShortcutHintProps,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = ShortcutHintRenderState {};
        let label_fg = self.label_fg(&state, theme);
        let kbd_bg = self.kbd_bg(&state, theme);
        let kbd_fg = self.kbd_fg(&state, theme);
        let bd = self.border(&state, theme);
        let px_h = self.padding_x(&state, theme);
        let px_v = self.padding_y(&state, theme);
        let key_g = self.key_gap(&state, theme);
        let label_g = self.label_gap(&state, theme);
        let family = self.font_family(&state, theme);

        let mut keys_row = gpui::div().flex().flex_row().items_center().gap(key_g);
        for key in &props.keys {
            keys_row = keys_row.child(
                gpui::div()
                    .bg(kbd_bg)
                    .text_color(kbd_fg)
                    .border(px(BRUTAL_BORDER_WIDTH))
                    .border_color(bd)
                    .rounded(px(BRUTAL_RADIUS))
                    .px(px_h)
                    .py(px_v)
                    .font_family(family.clone())
                    .text_size(px(12.))
                    .child(key.clone()),
            );
        }

        gpui::div()
            .id(props.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(label_g)
            .child(
                gpui::div()
                    .text_color(label_fg)
                    .font_family(family.clone())
                    .text_size(px(12.))
                    .child(props.label.clone()),
            )
            .child(keys_row)
    }
}

// =====================================================================
// Icon
// =====================================================================

pub use yororen_ui_core::renderer::icon::{IconRenderState, IconRenderer};

pub struct BrutalIconRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalIconRenderer {
    pub fn size(&self, state: &IconRenderState, theme: &Theme) -> Pixels {
        if state.has_custom_size {
            return px(0.0);
        }
        // Brutalism prefers slightly chunkier icons so they read
        // well next to the thick borders. Fallback to a higher
        // default than the system theme (16 vs 14).
        px(theme
            .get_number("tokens.control.icon.default_size")
            .or_else(|| theme.get_number("tokens.sizes.icon_md"))
            .unwrap_or(16.0) as f32)
    }

    pub fn color(&self, state: &IconRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_color {
            return BRUTAL_BORDER;
        }
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
}

impl IconRenderer for BrutalIconRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::icon::IconProps,
        cx: &App,
    ) -> gpui::AnyElement {
        let theme = cx.theme();
        let state = IconRenderState {
            has_custom_color: props.color.is_some(),
            has_custom_size: props.size.is_some(),
        };
        let path: SharedString = match &props.source {
            IconSource::Builtin(name) => format!("icons/{name}.svg").into(),
            IconSource::Resource(path) => path.clone(),
        };
        let size = props.size.unwrap_or_else(|| self.size(&state, theme));
        let color = props.color.unwrap_or_else(|| self.color(&state, theme));
        gpui::svg()
            .path(path)
            .size(size)
            .id(props.id.clone())
            .text_color(color)
            .into_any_element()
    }
}

// =====================================================================
// Text
// =====================================================================

pub use yororen_ui_core::renderer::text::{TextRenderState, TextRenderer};

pub struct BrutalTextRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalTextRenderer {
    pub fn size(&self, state: &TextRenderState, theme: &Theme) -> Pixels {
        if state.has_custom_size {
            return px(0.0);
        }
        px(theme
            .get_number("tokens.control.text.default_size")
            .or_else(|| theme.get_number("tokens.typography.font_size_md"))
            .unwrap_or(14.0) as f32)
    }

    pub fn color(&self, state: &TextRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_color {
            return BRUTAL_BORDER;
        }
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }

    pub fn family(&self, _state: &TextRenderState, theme: &Theme) -> SharedString {
        // Brutalism is mono-by-default. Fall back to the
        // brutalism font constant if the theme omits the
        // typography family path.
        theme
            .get_string("tokens.typography.family_default")
            .unwrap_or(BRUTAL_FONT_FAMILY)
            .to_string()
            .into()
    }
}

impl TextRenderer for BrutalTextRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::text::TextProps,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = TextRenderState {
            has_custom_size: props.size.is_some(),
            has_custom_color: false,
        };
        gpui::div()
            .id(props.id.clone())
            .text_size(props.size.unwrap_or_else(|| self.size(&state, theme)))
            .text_color(self.color(&state, theme))
            .font_family(self.family(&state, theme))
            .child(props.text.clone())
    }
}

// =====================================================================
// Spacer
// =====================================================================

pub use yororen_ui_core::renderer::spacer::{SpacerRenderState, SpacerRenderer};

pub struct BrutalSpacerRenderer;

impl SpacerRenderer for BrutalSpacerRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::spacer::SpacerProps,
        _cx: &App,
    ) -> Stateful<Div> {
        // A spacer is invisible by definition — brutalism has
        // nothing extra to add, so we mirror the default
        // renderer's `flex_1()` behaviour. The caller can layer
        // explicit width / height on top via the returned
        // `Stateful<Div>`.
        gpui::div().id(props.id.clone()).flex_1()
    }
}

// End of shortcut-hint impl.
