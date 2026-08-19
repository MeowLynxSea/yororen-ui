//! `WinUIFormRenderer` — default `FormRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, Pixels, Styled, div};

use yororen_ui_core::headless::form::FormProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::form::{FormRenderState, FormRenderer};

pub struct WinUIFormRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIFormRenderer {
    pub fn gap(&self, _state: &FormRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.form.field_gap")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn label_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.secondary").unwrap_or_default()
    }
    pub fn error_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("status.error.bg").unwrap_or_default()
    }
    pub fn helper_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
}

impl FormRenderer for WinUIFormRenderer {
    fn compose(&self, _props: &FormProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = FormRenderState {};
        let g = self.gap(&state, theme);
        div().flex().flex_col().gap(g)
    }
}

pub fn arc_form<T: FormRenderer + 'static>(r: T) -> Arc<dyn FormRenderer> {
    Arc::new(r)
}
