//! `WinUIPanelRenderer` — default `PanelRenderer` impl.
//!
//! The `Panel` component (defined in
//! `yororen_ui_core::headless::panel`) is the visual "card"
//! primitive that [`Modal`](yororen_ui_core::headless::modal::ModalProps)
//! and other dialog components compose. It carries a renderer
//! trait that themes override via the `RendererRegistry`.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div, px};

use yororen_ui_core::headless::panel::PanelProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::panel::{PanelRenderState, PanelRenderer};

pub struct WinUIPanelRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIPanelRenderer {
    pub fn bg(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.card_bg")
            .or_else(|| theme.get_color("surface.raised"))
            .unwrap_or_default()
    }
    pub fn border(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.card_stroke")
            .or_else(|| theme.get_color("border.default"))
            .unwrap_or_default()
    }
    pub fn padding(&self, _state: &PanelRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32,
        ))
    }
    pub fn border_radius(&self, _state: &PanelRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.panel.radius")
                .or_else(|| theme.get_number("tokens.radii.sm"))
                .unwrap_or(4.0) as f32,
        )
    }
    pub fn shadow_alpha(&self, _state: &PanelRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_2").unwrap_or_default().alpha
    }
    pub fn title_color(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
}

impl PanelRenderer for WinUIPanelRenderer {
    fn compose(&self, props: &PanelProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = PanelRenderState {
            has_custom_bg: props.has_custom_bg,
            has_custom_border: props.has_custom_border,
            has_custom_padding: props.has_custom_padding,
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
            .border_color(border)
            .p(pad.top)
            .rounded(r);
        if let Some(title) = &props.title {
            el = el.child(div().text_color(title_fg).pb(px(6.)).child(title.clone()));
        }
        el
    }
}

pub fn arc_panel<T: PanelRenderer + 'static>(r: T) -> Arc<dyn PanelRenderer> {
    Arc::new(r)
}
