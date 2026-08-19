//! `WinUITagRenderer` — default `TagRenderer` impl.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, FontWeight, Hsla, InteractiveElement, ParentElement, Pixels,
    StatefulInteractiveElement, Styled, div,
};

use yororen_ui_core::headless::tag::TagProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::tag::{TagRenderState, TagRenderer};

pub struct WinUITagRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUITagRenderer {
    pub fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.get_color("action.primary.bg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.bg").unwrap_or_default()
        }
    }

    pub fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else if state.has_custom_tone {
            theme.get_color("content.on_status").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.fg").unwrap_or_default()
        }
    }

    pub fn min_height(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.tag.min_height")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn padding_x(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32)
    }

    pub fn font_size(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_xs")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn font_weight(&self, _state: &TagRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_medium")
                .unwrap_or(500.0) as f32,
        )
    }

    pub fn border_radius(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
    }

    pub fn close_size(&self, _state: &TagRenderState, _theme: &Theme) -> Pixels {
        gpui::px(16.)
    }

    pub fn close_hover_bg(&self, _state: &TagRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
}

impl TagRenderer for WinUITagRenderer {
    fn compose(&self, props: &TagProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TagRenderState {
            selected: props.selected,
            has_custom_tone: false,
            closable: props.closable,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let h = self.min_height(&state, theme);
        let p = self.padding_x(&state, theme);
        let fs = self.font_size(&state, theme);
        let fw = self.font_weight(&state, theme);
        let r = self.border_radius(&state, theme);
        let mut el = div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .min_h(h)
            .px(p)
            .text_size(fs)
            .font_weight(fw)
            .rounded(r)
            .gap(p / 2.)
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else if props.on_click.is_some() {
                CursorStyle::PointingHand
            } else {
                CursorStyle::Arrow
            })
            .child(props.label.clone());
        if props.closable {
            let close_size = self.close_size(&state, theme);
            // `on_click` lives on `StatefulInteractiveElement`,
            // which requires the Div to have an id. Derive a
            // stable, unique id from the tag's own id so the
            // close button gets a distinct identity.
            let close_id: gpui::ElementId = match &props.id {
                gpui::ElementId::Name(name) => {
                    let mut s = name.to_string();
                    s.push_str("__close");
                    s.into()
                }
                _ => "tag_close".into(),
            };
            let mut close_btn = div()
                .id(close_id)
                .flex()
                .items_center()
                .justify_center()
                .size(close_size)
                .rounded(close_size / 2.)
                .cursor(gpui::CursorStyle::PointingHand)
                .child("×");
            // Wire the close callback (mirrors how `apply`
            // wires `on_click` on the tag itself). We
            // `stop_propagation` so a click on `×` doesn't
            // also fire the tag's select-toggle on_click —
            // a click on the close affordance should close,
            // not toggle.
            if !props.disabled
                && let Some(f) = props.on_close.clone()
            {
                close_btn = close_btn.on_click(move |ev, window, cx: &mut gpui::App| {
                    cx.stop_propagation();
                    f(ev, window, cx);
                });
            }
            el = el.child(close_btn);
        }
        el
    }
}

pub fn arc_tag<T: TagRenderer + 'static>(r: T) -> Arc<dyn TagRenderer> {
    Arc::new(r)
}
