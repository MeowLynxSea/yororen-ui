//! `WinUITooltipRenderer` — default `TooltipRenderer` impl.
//!
//! Composes the tooltip trigger and attaches gpui's built-in
//! `hoverable_tooltip` so the tooltip panel is shown/hidden by
//! the platform on hover. This avoids relying on the parent view
//! re-rendering in response to `TooltipState` changes, which does
//! not work reliably when the tooltip lives inside a virtual-list
//! row. The core still owns the tooltip text / state; the renderer
//! only decides how the floating panel looks.

use std::sync::Arc;

use gpui::{
    App, AppContext, Context, Div, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    Render, StatefulInteractiveElement, Styled, Window, div,
};

use yororen_ui_core::headless::tooltip::TooltipProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::tooltip::{TooltipRenderState, TooltipRenderer};

pub struct WinUITooltipRenderer;

/// View rendered by gpui's `hoverable_tooltip` builder.
/// It captures the computed theme values so the tooltip panel
/// matches the renderer's tokens.
struct TooltipView {
    text: String,
    bg: Hsla,
    fg: Hsla,
    border: Hsla,
    pad_top: Pixels,
    font_size: Pixels,
    border_radius: Pixels,
    max_width: Pixels,
}

impl Render for TooltipView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(self.bg)
            .text_color(self.fg)
            .border_1()
            .border_color(self.border)
            .p(self.pad_top)
            .text_size(self.font_size)
            .rounded(self.border_radius)
            .max_w(self.max_width)
            .child(self.text.clone())
    }
}

// Inherent helpers — *not* part of the `TooltipRenderer` trait surface.
impl WinUITooltipRenderer {
    pub fn bg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.popover")
            .or_else(|| theme.get_color("action.neutral.bg"))
            .unwrap_or_default()
    }
    pub fn fg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn padding(&self, _state: &TooltipRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
        )
    }
    pub fn font_size(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_sm")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
}

impl TooltipRenderer for WinUITooltipRenderer {
    fn compose(&self, props: &mut TooltipProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TooltipRenderState {
            has_custom_bg: props.has_custom_bg,
            has_custom_fg: props.has_custom_fg,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let border = theme
            .get_color("winui.flyout_stroke")
            .or_else(|| theme.get_color("border.default"))
            .unwrap_or_default();
        let pad = self.padding(&state, theme);
        let fs = self.font_size(&state, theme);
        let r = self.border_radius(&state, theme);
        let max_w = gpui::px(
            theme
                .get_number("tokens.control.tooltip.max_width")
                .unwrap_or(240.0) as f32,
        );

        let mut outer = div().flex().flex_col().items_start();

        // 1) Trigger — wrap it in a `Stateful<Div>` so we can attach
        //    gpui's `hoverable_tooltip`. The tooltip content is built
        //    from the core-provided text and renderer tokens.
        if let Some(t) = props.trigger.take() {
            let text = props.text.clone();
            let trigger_id = format!("{}-trigger", props.id);
            outer = outer.child(div().id(trigger_id).child(t).hoverable_tooltip(
                move |_window, cx| {
                    cx.new(|_cx| TooltipView {
                        text: text.clone(),
                        bg,
                        fg,
                        border,
                        pad_top: pad.top,
                        font_size: fs,
                        border_radius: r,
                        max_width: max_w,
                    })
                    .into()
                },
            ));
        }

        outer
    }
}

pub fn arc_tooltip<T: TooltipRenderer + 'static>(r: T) -> Arc<dyn TooltipRenderer> {
    Arc::new(r)
}
