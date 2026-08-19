//! `WinUIProgressBarRenderer` — default `ProgressBarRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::progress::ProgressBarProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::progress::{ProgressBarRenderState, ProgressBarRenderer};

pub struct WinUIProgressBarRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIProgressBarRenderer {
    pub fn track(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.subtle_fill")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default()
    }

    pub fn fill(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.accent")
            .or_else(|| theme.get_color("action.primary.bg"))
            .unwrap_or_default()
    }

    pub fn height(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.progress.bar_default_h")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn border_color(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }

    pub fn border_radius(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
    }
}

impl ProgressBarRenderer for WinUIProgressBarRenderer {
    fn compose(&self, props: &ProgressBarProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ProgressBarRenderState {
            indeterminate: props.indeterminate,
            has_custom_height: props.has_custom_height,
        };
        let track = self.track(&state, theme);
        let fill = self.fill(&state, theme);
        let h = self.height(&state, theme);
        let r = self.border_radius(&state, theme);
        // Compute fill ratio, clamped to [0, 1].
        let ratio = if props.indeterminate || props.max <= 0.0 {
            0.0
        } else {
            (props.value / props.max).clamp(0.0, 1.0)
        };
        div()
            .flex()
            .flex_col()
            .w_full()
            .h(h)
            .bg(track)
            .rounded(r)
            .child(div().h_full().w(gpui::relative(ratio)).bg(fill).rounded(r))
    }
}

pub fn arc_progress_bar<T: ProgressBarRenderer + 'static>(r: T) -> Arc<dyn ProgressBarRenderer> {
    Arc::new(r)
}
