//! `WinUIIconRenderer` — default `IconRenderer` impl.
//!
//! Resolves builtin icon names to `icons/<name>.svg`, applies the
//! caller's size / color, and falls back to theme tokens when either
//! is omitted.

use std::sync::Arc;

use gpui::{AnyElement, App, Hsla, InteractiveElement, IntoElement, Pixels, Styled, svg};

use yororen_ui_core::headless::icon::{IconProps, IconSource};
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::icon::{IconRenderState, IconRenderer};

pub struct WinUIIconRenderer;

impl WinUIIconRenderer {
    pub fn size(&self, state: &IconRenderState, theme: &Theme) -> Pixels {
        if state.has_custom_size {
            return gpui::px(0.0);
        }
        gpui::px(theme.get_number("tokens.sizes.icon_md").unwrap_or(14.0) as f32)
    }

    pub fn color(&self, state: &IconRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_color {
            return gpui::rgb(0x0A0A0A).into();
        }
        theme
            .get_color("content.primary")
            .unwrap_or_else(|| gpui::rgb(0x0A0A0A).into())
    }
}

impl IconRenderer for WinUIIconRenderer {
    fn compose(&self, props: &IconProps, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let state = IconRenderState {
            has_custom_color: props.color.is_some(),
            has_custom_size: props.size.is_some(),
        };

        let path = match &props.source {
            IconSource::Builtin(name) => gpui::SharedString::from(format!("icons/{name}.svg")),
            IconSource::Resource(path) => path.clone(),
        };
        let size = props.size.unwrap_or_else(|| self.size(&state, theme));
        let color = props.color.unwrap_or_else(|| self.color(&state, theme));

        svg()
            .path(path)
            .size(size)
            .id(props.id.clone())
            .text_color(color)
            .into_any_element()
    }
}

pub fn arc_icon<T: IconRenderer + 'static>(r: T) -> Arc<dyn IconRenderer> {
    Arc::new(r)
}
