//! `ToastRenderer` — visual side of `Toast`.

use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::renderer::spec::Edges;
pub use yororen_ui_core::renderer::toast::{ToastRenderState, ToastRenderer};
use yororen_ui_core::theme::Theme;

pub struct WinUIToastRenderer;

impl ToastRenderer for WinUIToastRenderer {
    fn bg(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or_default()
    }
    fn fg(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    fn padding(&self, _state: &ToastRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(
                theme
                    .get_number("tokens.control.toast.horizontal_padding")
                    .unwrap_or(16.0) as f32,
            ),
            gpui::px(
                theme
                    .get_number("tokens.control.toast.vertical_padding")
                    .unwrap_or(12.0) as f32,
            ),
        )
    }
    fn border_radius(&self, _state: &ToastRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn border(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn shadow_alpha(&self, _state: &ToastRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_2").unwrap_or_default().a
    }
}

pub fn arc_toast<T: ToastRenderer + 'static>(r: T) -> Arc<dyn ToastRenderer> {
    Arc::new(r)
}
