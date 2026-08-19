//! `WinUIDividerRenderer` — default `DividerRenderer` impl.

use std::sync::Arc;

use gpui::{AlignSelf, App, Div, Hsla, Pixels, Styled, div};

use yororen_ui_core::headless::divider::DividerProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::divider::{DividerRenderState, DividerRenderer};

pub struct WinUIDividerRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIDividerRenderer {
    pub fn color(&self, _state: &DividerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.divider").unwrap_or_default()
    }

    pub fn thickness(&self, _state: &DividerRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.divider.thickness")
                .unwrap_or(1.0) as f32,
        )
    }
}

impl DividerRenderer for WinUIDividerRenderer {
    fn compose(&self, props: &DividerProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DividerRenderState {
            horizontal: props.horizontal,
        };
        let color = self.color(&state, theme);
        let thickness = self.thickness(&state, theme);
        let mut el = div().bg(color).flex_shrink_0();
        if props.horizontal {
            // `min_h` guarantees the line stays visible inside a flex
            // container whose main axis is vertical — `h` alone can be
            // collapsed by flex-basis sizing when the item has no
            // content to derive a base size from.
            // `align_self: stretch` forces the cross-axis size (width
            // in a flex column) to match the container, so the divider
            // spans the full section even when the parent's width is
            // content-derived and `w_full()` would resolve to zero.
            el.style().align_self = Some(AlignSelf::Stretch);
            el = el.w_full().h(thickness).min_h(thickness);
        } else {
            el.style().align_self = Some(AlignSelf::Stretch);
            el = el.h_full().w(thickness).min_w(thickness);
        }
        el
    }
}

pub fn arc_divider<T: DividerRenderer + 'static>(r: T) -> Arc<dyn DividerRenderer> {
    Arc::new(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/winui-light.json");
        Theme::from_json(json).expect("winui-light.json is valid")
    }

    #[test]
    fn color_reads_border_divider_path() {
        let theme = fixture();
        let r = WinUIDividerRenderer;
        let state = DividerRenderState::default();
        assert_eq!(
            r.color(&state, &theme),
            theme.get_color("border.divider").unwrap_or_default(),
        );
    }

    #[test]
    fn thickness_reads_control_divider_thickness_path() {
        let theme = fixture();
        let r = WinUIDividerRenderer;
        let state = DividerRenderState::default();
        let expected = theme
            .get_number("tokens.control.divider.thickness")
            .unwrap_or(1.0) as f32;
        assert_eq!(r.thickness(&state, &theme), gpui::px(expected));
    }

    #[test]
    fn missing_paths_yield_zero_color() {
        let theme = Theme::from_value(serde_json::json!({}));
        let r = WinUIDividerRenderer;
        let state = DividerRenderState::default();
        let _ = r.color(&state, &theme);
        let _ = r.thickness(&state, &theme);
    }
}
