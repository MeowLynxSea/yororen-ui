//! `WinUIDisclosureRenderer` — default `DisclosureRenderer` impl.
//!
//! Follows the reference `WinExpander` header anatomy: a 48px-tall
//! trigger row with 16px side padding, the chevron living inside a
//! 32×32 (4px-radius) box that takes the subtle hover fill, and the
//! glyph switching between the collapsed (right) and expanded
//! (down) Fluent arrows. The caller appends the expanded body as a
//! child after `.render(cx)`; the headless layer wires `on_toggle`
//! via `.apply()`.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, Hsla, InteractiveElement, ParentElement, Pixels, Styled, div, px,
};

use yororen_ui_core::headless::disclosure::DisclosureProps;
use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::headless::icon::IconSource;
use yororen_ui_core::theme::Theme;

use crate::animation::{AnimatedStateElement, control_config, lerp_hsla, set_interaction_hovered};

pub use yororen_ui_core::renderer::disclosure::{DisclosureRenderState, DisclosureRenderer};

pub struct WinUIDisclosureRenderer;

impl WinUIDisclosureRenderer {
    pub fn bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.subtle_fill_secondary")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default()
    }
    /// Chevron-box hover fill (the 32×32 square, not the whole row).
    pub fn chevron_hover_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.subtle_fill_secondary")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default()
    }
    pub fn fg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn border_radius(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
    }
    /// Expander header height: 48px in the reference.
    pub fn min_height(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.disclosure.min_height")
                .unwrap_or(48.0) as f32,
        )
    }
    pub fn chevron_size(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.disclosure.chevron_size")
                .unwrap_or(12.0) as f32,
        )
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
        let min_h = self.min_height(&state, theme);
        let chevron_size = self.chevron_size(&state, theme);
        let chevron_hover_bg = self.chevron_hover_bg(&state, theme);
        let chevron_fg = theme
            .get_color("winui.text_secondary")
            .or_else(|| theme.get_color("content.secondary"))
            .unwrap_or(fg);

        let mut container = div()
            .relative()
            .flex()
            .flex_col()
            .rounded(r)
            .text_color(fg)
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            });

        if !props.disabled {
            let id = props.id.clone();
            container
                .interactivity()
                .on_hover(move |hovered, _win, cx| {
                    set_interaction_hovered(cx, id.clone(), *hovered);
                });
        }

        let config = control_config(theme);
        let fill = AnimatedStateElement::new(
            (props.id.clone(), "fill"),
            props.id.clone(),
            false,
            div().absolute().inset_0().rounded(r),
            config.clone(),
            move |d: Div, hover, _pressed, _checked| d.bg(lerp_hsla(bg, hover_bg, hover)),
        );
        container = container.child(fill);

        // Chevron inside its own 32×32 subtle-hover box.
        let chevron_id = (props.id.clone(), "chevron").into();
        let chevron_icon = IconProps {
            id: (chevron_id, "icon").into(),
            source: IconSource::Builtin(
                if props.open {
                    "arrow-down"
                } else {
                    "arrow-right"
                }
                .into(),
            ),
            size: Some(chevron_size),
            color: Some(chevron_fg),
        }
        .render(cx);
        let chevron_box = AnimatedStateElement::new(
            (props.id.clone(), "chevron-fill"),
            props.id.clone(),
            false,
            div()
                .size(px(32.0))
                .rounded(px(4.0))
                .flex()
                .items_center()
                .justify_center()
                .child(chevron_icon),
            config,
            move |d: Div, hover, _pressed, _checked| {
                d.bg(lerp_hsla(
                    gpui::hsla(0.0, 0.0, 0.0, 0.0),
                    chevron_hover_bg,
                    hover,
                ))
            },
        );

        // 48px header row, 16px side padding, 16px gap.
        container.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(16.0))
                .min_h(min_h)
                .pl(px(16.0))
                .pr(px(16.0))
                .child(chevron_box)
                .child(props.title.clone()),
        )
    }
}

pub fn arc_disclosure<T: DisclosureRenderer + 'static>(r: T) -> Arc<dyn DisclosureRenderer> {
    Arc::new(r)
}
