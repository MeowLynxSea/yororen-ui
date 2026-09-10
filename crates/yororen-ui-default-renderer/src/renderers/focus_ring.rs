//! `TokenFocusRingRenderer` — default `FocusRingRenderer` impl.

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, InteractiveElement, Pixels, Stateful, Styled, div, point};

use yororen_ui_core::headless::focus_ring::FocusRingProps;
pub use yororen_ui_core::renderer::focus_ring::{FocusRingRenderState, FocusRingRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenFocusRingRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenFocusRingRenderer {
    pub fn color(&self, _state: &FocusRingRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }

    pub fn width(&self, _state: &FocusRingRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.focus_ring.width")
                .unwrap_or(2.0) as f32,
        )
    }

    /// Default corner radius for the ring. Reads
    /// `tokens.control.focus_ring.radius`, falling back to
    /// `tokens.radii.lg`. The ring is drawn *outside* the wrapped
    /// element via a positive `spread_radius`, so its outer corner
    /// is `child_radius + width`. Using `radii.lg` (8) rather than
    /// `radii.md` (6) keeps the ring concentric with the default
    /// button (radius 6 + width 2 = 8). Falls back to 8.0 if no
    /// token is set.
    pub fn border_radius(&self, _state: &FocusRingRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.focus_ring.radius")
                .or_else(|| theme.get_number("tokens.radii.lg"))
                .unwrap_or(8.0) as f32,
        )
    }
}

impl FocusRingRenderer for TokenFocusRingRenderer {
    fn compose(&self, props: &FocusRingProps, cx: &gpui::App) -> Stateful<gpui::Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = FocusRingRenderState {
            has_custom_color: props.has_custom_color,
        };
        let color = self.color(&state, theme);
        let width = self.width(&state, theme);
        let radius = self.border_radius(&state, theme);
        // Draw a ring via box-shadow with no offset and a positive
        // spread radius equal to the ring width. The wrapper div
        // has its own `rounded(radius)` so the shadow inherits the
        // same corner curve (box-shadow always follows the host
        // element's border-radius), producing a ring whose corners
        // match the wrapped content.
        div()
            .id(props.id.clone())
            .track_focus(&props.focus_handle)
            .rounded(radius)
            .shadow(vec![BoxShadow {
                color,
                offset: point(gpui::px(0.), gpui::px(0.)),
                blur_radius: gpui::px(0.),
                spread_radius: width,
                inset: false,
            }])
    }
}

pub fn arc_focus_ring<T: FocusRingRenderer + 'static>(r: T) -> Arc<dyn FocusRingRenderer> {
    Arc::new(r)
}
