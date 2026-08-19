//! `WinUIDisclosureRenderer` — default `DisclosureRenderer` impl.
//!
//! Paints a flex-column container with a subtle hover background
//! and a chevron + title trigger row. The caller appends the
//! expanded body as a child after `.render(cx)`. The headless
//! layer wires `on_toggle` via `.apply()`.

use std::sync::Arc;
use std::time::Duration;

use gpui::{App, CursorStyle, Div, Hsla, InteractiveElement, ParentElement, Pixels, Styled, div};

use yororen_ui_core::animation::AnimationConfig;
use yororen_ui_core::headless::disclosure::DisclosureProps;
use yororen_ui_core::theme::Theme;

use crate::animation::{AnimatedStateElement, lerp_hsla, set_interaction_hovered};

pub use yororen_ui_core::renderer::disclosure::{DisclosureRenderState, DisclosureRenderer};

pub struct WinUIDisclosureRenderer;

impl WinUIDisclosureRenderer {
    pub fn bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn fg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn border_radius(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
    }
    pub fn gap(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32)
    }
}

impl DisclosureRenderer for WinUIDisclosureRenderer {
    fn compose(&self, props: &DisclosureProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DisclosureRenderState { open: props.open };
        let bg = self.bg(&state, theme);
        let hover_bg = self.hover_bg(&state, theme);
        let fg = self.fg(&state, theme);
        let r = self.border_radius(&state, theme);
        let gap = self.gap(&state, theme);
        let chev_str = if props.open { "▼" } else { "▶" };

        let mut container = div()
            .relative()
            .flex()
            .flex_col()
            .gap(gap)
            .rounded(r)
            .text_color(fg)
            .cursor(CursorStyle::PointingHand);

        if !props.disabled {
            let id = props.id.clone();
            container
                .interactivity()
                .on_hover(move |hovered, _win, cx| {
                    set_interaction_hovered(cx, id.clone(), *hovered);
                });
        }

        let config = AnimationConfig::default().with_duration(Duration::from_millis(100));
        let fill = AnimatedStateElement::new(
            (props.id.clone(), "fill"),
            props.id.clone(),
            false,
            div().absolute().inset_0().rounded(r),
            config,
            move |d: Div, hover, _pressed, _checked| d.bg(lerp_hsla(bg, hover_bg, hover)),
        );
        container = container.child(fill);

        container.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(gpui::px(6.0))
                .child(chev_str)
                .child(props.title.clone()),
        )
    }
}

pub fn arc_disclosure<T: DisclosureRenderer + 'static>(r: T) -> Arc<dyn DisclosureRenderer> {
    Arc::new(r)
}
