//! yororen-ui Variant Showcase Demo
//!
//! Three side-by-side buttons showing that swapping
//! `ActionVariantKind` (Neutral / Primary / Danger) on the
//! *same* `headless::button` factory re-routes the
//! `ButtonRenderer` through different `action.<key>.*` token
//! paths in the theme JSON.
//!
//! A fourth button demonstrates the `apply` escape hatch:
//! when you need a shape that `default_render` doesn't
//! provide (e.g. a fixed-size pill), you can read the same
//! renderer tokens by hand and compose your own `div`,
//! then wire it back to the headless button via `apply(...)`.

use gpui::{
    Context, Hsla, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, hsla, px,
};
use yororen_ui::ActionVariantKind;
use yororen_ui::ActiveTheme;
use yororen_ui::Theme;
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use yororen_ui::headless::layout::{Inset, Spacing, column, row};
use yororen_ui::renderer::ButtonRenderState;

/// Inherent helper renderer for the variant_showcase demo.
/// Reads the same `action.<variant>.{bg,fg,hover_bg,active_bg}`
/// tokens as the default renderer. Lives outside the trait so
/// the demo can pull the colour palette without going through
/// the trait's `compose` method.
///
/// The fallbacks are explicit visible colours: a fully
/// transparent `Hsla::default()` would make the pill (built
/// from these tokens) disappear, defeating the demo.
struct DemoButtonRenderer;

impl DemoButtonRenderer {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or_else(|| fallback_for(state.variant, field, state.disabled))
    }
    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        let field = if state.disabled { "disabled_fg" } else { "fg" };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or_else(|| fallback_for(state.variant, field, state.disabled))
    }
    fn hover_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "hover_bg"
        };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or_else(|| fallback_for(state.variant, field, state.disabled))
    }
    fn active_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "active_bg"
        };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or_else(|| fallback_for(state.variant, field, state.disabled))
    }
}

/// Per-variant fallback palette. Used when a theme JSON omits
/// one of the `action.<variant>.{bg,fg,hover_bg,active_bg,
/// disabled_bg,disabled_fg}` keys — the default renderer's
/// `unwrap_or_default()` would produce a fully transparent
/// `Hsla` and the button would visually disappear. These
/// colours match the bundled `system-light` / `system-dark`
/// palettes roughly so the demo remains legible.
struct FallbackPalette {
    bg: Hsla,
    fg: Hsla,
    hover_bg: Hsla,
    active_bg: Hsla,
    disabled_bg: Hsla,
    disabled_fg: Hsla,
}

fn fallback_palette(variant: ActionVariantKind) -> FallbackPalette {
    // A tiny constructor instead of a `static` because
    // `hsla()` is not a `const fn` (yet). Cheap enough —
    // a handful of float copies — and only called when a
    // theme omits an action key, which is the demo's whole
    // point.
    match variant {
        ActionVariantKind::Neutral => FallbackPalette {
            bg: hsla(0.0, 0.0, 0.95, 1.0),
            fg: hsla(0.0, 0.0, 0.1, 1.0),
            hover_bg: hsla(0.0, 0.0, 0.9, 1.0),
            active_bg: hsla(0.0, 0.0, 0.85, 1.0),
            disabled_bg: hsla(0.0, 0.0, 0.92, 1.0),
            disabled_fg: hsla(0.0, 0.0, 0.5, 1.0),
        },
        ActionVariantKind::Primary => FallbackPalette {
            bg: hsla(0.58, 0.7, 0.55, 1.0),
            fg: hsla(0.0, 0.0, 1.0, 1.0),
            hover_bg: hsla(0.58, 0.75, 0.6, 1.0),
            active_bg: hsla(0.58, 0.65, 0.5, 1.0),
            disabled_bg: hsla(0.58, 0.3, 0.7, 1.0),
            disabled_fg: hsla(0.0, 0.0, 0.9, 1.0),
        },
        ActionVariantKind::Danger => FallbackPalette {
            bg: hsla(0.0, 0.7, 0.5, 1.0),
            fg: hsla(0.0, 0.0, 1.0, 1.0),
            hover_bg: hsla(0.0, 0.75, 0.55, 1.0),
            active_bg: hsla(0.0, 0.65, 0.45, 1.0),
            disabled_bg: hsla(0.0, 0.3, 0.7, 1.0),
            disabled_fg: hsla(0.0, 0.0, 0.9, 1.0),
        },
    }
}

fn field_pick(palette: &FallbackPalette, disabled: bool, field: &'static str) -> Hsla {
    if disabled {
        match field {
            "bg" | "hover_bg" | "active_bg" => palette.disabled_bg,
            "fg" => palette.disabled_fg,
            // `field` comes from a fixed set in
            // `DemoButtonRenderer::{bg,fg,hover_bg,active_bg}`
            // so this arm is unreachable. Keep it as a
            // safety net for typos.
            _ => palette.disabled_bg,
        }
    } else {
        match field {
            "bg" => palette.bg,
            "fg" => palette.fg,
            "hover_bg" => palette.hover_bg,
            "active_bg" => palette.active_bg,
            // `disabled_bg` / `disabled_fg` are only used
            // when the button is disabled. The "not
            // disabled" branch falls back to `bg` if a
            // caller ever asks for them by mistake.
            "disabled_bg" => palette.disabled_bg,
            "disabled_fg" => palette.disabled_fg,
            _ => palette.bg,
        }
    }
}

fn fallback_for(variant: ActionVariantKind, field: &'static str, disabled: bool) -> Hsla {
    field_pick(&fallback_palette(variant), disabled, field)
}

pub struct VariantApp;

impl VariantApp {
    pub fn new() -> Self {
        Self
    }
}

impl Render for VariantApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Paint the root with the theme's surface color so the
        // window background is opaque on all platforms (notably
        // Windows, where an unset background shows through as
        // transparent).
        let surface = cx
            .theme()
            .get_color("surface.base")
            .unwrap_or_else(|| hsla(0.0, 0.0, 0.98, 1.0));

        // Read Primary tokens once for the override example
        // below. Scoped to a block so the immutable borrow of
        // `cx` is released before we call the headless factories
        // (which need `&mut App`).
        let (primary_bg, primary_fg, primary_hover_bg, primary_active_bg) = {
            let theme: &Theme = cx.theme();
            let r = DemoButtonRenderer;
            let state = ButtonRenderState {
                variant: ActionVariantKind::Primary,
                ..Default::default()
            };
            (
                r.bg(&state, theme),
                r.fg(&state, theme),
                r.hover_bg(&state, theme),
                r.active_bg(&state, theme),
            )
        };

        // === Three "default_render" buttons: same factory,
        // different variant. Only `ButtonRenderState.variant`
        // changes, which re-routes the renderer to a different
        // `action.<key>.*` token path. ===
        let neutral = button("neutral-btn", cx)
            .variant(ActionVariantKind::Neutral)
            .render(cx)
            .child("Neutral");

        let primary = button("primary-btn", cx)
            .variant(ActionVariantKind::Primary)
            .render(cx)
            .child("Primary");

        let danger = button("danger-btn", cx)
            .variant(ActionVariantKind::Danger)
            .render(cx)
            .child("Danger");

        // === Escape hatch: same Primary tokens, but a shape
        // that `default_render` doesn't expose — fixed 220×56
        // pill. We pull the theme colors from the renderer by
        // hand, build our own `div`, and wire a11y/click back
        // through `apply`. The hover/active overrides re-use
        // the renderer's own hover/active tokens, so the
        // visual feedback stays theme-driven. ===
        let pill = button("pill-btn", cx)
            .on_click(|_, _, _| {})
            .apply(
                div()
                    .bg(primary_bg)
                    .text_color(primary_fg)
                    .w(px(220.))
                    .h(px(56.))
                    .rounded(px(28.))
                    .cursor(gpui::CursorStyle::PointingHand)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("Pill - custom shape"),
            )
            .hover(|s| s.bg(primary_hover_bg))
            .active(|s| s.bg(primary_active_bg));

        column("variant-root", cx)
            .w_full()
            .h_full()
            .p(Inset::Lg)
            .gap(Spacing::Md)
            .child(label("title", "Variant showcase", cx).render(cx))
            .child(
                label(
                    "blurb",
                    "Same headless::button, different ButtonRenderState.variant → different action.<key>.* paths from the JSON.",
                    cx,
                )
                .render(cx),
            )
            .child(
                row("neutral-row", cx)
                    .gap(Spacing::Sm)
                    .child(label("n", "Neutral (default_render):", cx).render(cx))
                    .child(neutral)
                    .render(cx),
            )
            .child(
                row("primary-row", cx)
                    .gap(Spacing::Sm)
                    .child(label("p", "Primary (default_render):", cx).render(cx))
                    .child(primary)
                    .render(cx),
            )
            .child(
                row("danger-row", cx)
                    .gap(Spacing::Sm)
                    .child(label("d", "Danger (default_render):", cx).render(cx))
                    .child(danger)
                    .render(cx),
            )
            .child(
                row("override-row", cx)
                    .gap(Spacing::Sm)
                    .child(
                        label(
                            "o",
                            "Override (apply + custom shape):",
                            cx,
                        )
                        .render(cx),
                    )
                    .child(pill)
                    .render(cx),
            )
            .render(cx)
            .bg(surface)
    }
}
