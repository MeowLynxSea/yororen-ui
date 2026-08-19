//! `WinUIButtonGroupRenderer` — default `ButtonGroupRenderer` impl.
//!
//! Produces a **segmented control** look by default:
//! - No gap between children (they're attached).
//! - A single border wraps the whole group.
//! - `overflow_hidden()` + outer `rounded()` clips the
//!   container to a rounded rectangle.
//! - Each child has its rounded corners stripped to 0, then
//!   the first child gets its leading corners re-rounded and
//!   the last child gets its trailing corners re-rounded.
//!   Middle children stay fully square (the container's
//!   `overflow_hidden()` would clip them to a rectangle
//!   anyway, but stripping them explicitly gives a cleaner
//!   antialiasing result and avoids a faint "double-curve"
//!   where two rounded buttons meet).
//!
//! Reads:
//! - `tokens.control.button_group.gap` — gap between children
//!   in **detached** mode (default: `2`).
//! - `tokens.control.button.radius` — the corner radius the
//!   outer buttons inherit (so the segmented control's
//!   silhouette matches standalone buttons). Falls back to
//!   `tokens.radii.md` and then to `6`.
//! - `border.default` — the shared border colour.

use std::sync::Arc;

use gpui::{App, Div, Hsla, InteractiveElement, ParentElement, Pixels, Stateful, Styled, div, px};

use yororen_ui_core::headless::button_group::ButtonGroupOrientation;
use yororen_ui_core::headless::button_group::ButtonGroupProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::button_group::{ButtonGroupRenderState, ButtonGroupRenderer};

pub struct WinUIButtonGroupRenderer;

// Inherent helpers — *not* part of the trait surface. They
// exist so other renderers can share the same palette
// lookups by depending on `WinUIButtonGroupRenderer` directly,
// and so unit tests can assert on the token path.
impl WinUIButtonGroupRenderer {
    /// Gap between children in **detached** mode. In attached
    /// (segmented) mode the gap is always 0.
    pub fn gap(&self, _state: &ButtonGroupRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.button_group.gap")
            .unwrap_or(2.0) as f32
    }

    /// Corner radius the first / last button inherit. Matches
    /// the standalone button radius so the segmented control
    /// silhouette is consistent with regular buttons.
    pub fn radius(&self, _state: &ButtonGroupRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.radius")
            .or_else(|| theme.get_number("tokens.radii.md"))
            .unwrap_or(6.0) as f32)
    }

    /// Border colour for the shared group border (only drawn
    /// in attached mode).
    pub fn border_color(&self, _state: &ButtonGroupRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
}

impl ButtonGroupRenderer for WinUIButtonGroupRenderer {
    fn compose(&self, props: ButtonGroupProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ButtonGroupRenderState {
            orientation: props.orientation,
            attached: props.attached,
        };
        let n = props.children.len();
        let id = props.id;
        let children = props.children;

        // Container: flex row/column. The border, radius, and
        // overflow are only applied in attached mode — in
        // detached mode each child keeps its own styling.
        let mut container = match props.orientation {
            ButtonGroupOrientation::Horizontal => div().flex().flex_row().items_center(),
            ButtonGroupOrientation::Vertical => div().flex().flex_col().items_center(),
        };

        if props.attached && n > 0 {
            let radius = self.radius(&state, theme);
            let border = self.border_color(&state, theme);
            container = container
                .overflow_hidden()
                .rounded(radius)
                .border_1()
                .border_color(border);
        } else {
            let gap = px(self.gap(&state, theme));
            container = container.gap(gap);
        }

        // Process children: strip their own rounded corners and
        // re-add the outer corners to the first / last child.
        let mut iter = children.into_iter();
        for i in 0..n {
            let Some(mut child) = iter.next() else { break };
            if props.attached && n > 1 {
                let radius = self.radius(&state, theme);
                child = child.rounded(px(0.));
                if i == 0 {
                    // First child: keep the leading corners.
                    if props.orientation == ButtonGroupOrientation::Horizontal {
                        child = child.rounded_tl(radius).rounded_bl(radius);
                    } else {
                        child = child.rounded_tl(radius).rounded_tr(radius);
                    }
                } else if i + 1 == n {
                    // Last child: keep the trailing corners.
                    if props.orientation == ButtonGroupOrientation::Horizontal {
                        child = child.rounded_tr(radius).rounded_br(radius);
                    } else {
                        child = child.rounded_bl(radius).rounded_br(radius);
                    }
                }
                // Middle children stay square.
            }
            container = container.child(child);
        }

        container.id(id)
    }
}

pub fn arc_button_group<T: ButtonGroupRenderer + 'static>(r: T) -> Arc<dyn ButtonGroupRenderer> {
    Arc::new(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yororen_ui_core::theme::Theme;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/winui-dark.json");
        Theme::from_json(json).expect("winui-dark.json is valid")
    }

    #[test]
    fn gap_reads_button_group_token() {
        let theme = fixture();
        let r = WinUIButtonGroupRenderer;
        let state = ButtonGroupRenderState {
            attached: false,
            ..Default::default()
        };
        let expected = theme
            .get_number("tokens.control.button_group.gap")
            .unwrap_or(2.0) as f32;
        assert_eq!(r.gap(&state, &theme), expected);
    }

    #[test]
    fn missing_gap_token_yields_default() {
        let theme = Theme::from_value(serde_json::json!({}));
        let r = WinUIButtonGroupRenderer;
        let state = ButtonGroupRenderState {
            attached: false,
            ..Default::default()
        };
        assert_eq!(r.gap(&state, &theme), 2.0);
    }

    #[test]
    fn radius_falls_back_to_md_then_6() {
        let theme = Theme::from_value(serde_json::json!({}));
        let r = WinUIButtonGroupRenderer;
        let state = ButtonGroupRenderState::default();
        assert_eq!(r.radius(&state, &theme), px(6.0));
    }

    #[test]
    fn radius_prefers_button_token() {
        let theme = Theme::from_value(serde_json::json!({
            "tokens": { "control": { "button": { "radius": 12.0 } } }
        }));
        let r = WinUIButtonGroupRenderer;
        let state = ButtonGroupRenderState::default();
        assert_eq!(r.radius(&state, &theme), px(12.0));
    }
}
