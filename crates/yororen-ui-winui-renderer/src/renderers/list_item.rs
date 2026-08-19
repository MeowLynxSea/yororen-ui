//! `WinUIListItemRenderer` — default `ListItemRenderer` impl.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, Hsla, InteractiveElement, ParentElement, Pixels, Styled, div, px,
};

use yororen_ui_core::headless::icon::{IconProps, IconSource};
use yororen_ui_core::headless::list_item::ListItemProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::list_item::{ListItemRenderState, ListItemRenderer};

pub struct WinUIListItemRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIListItemRenderer {
    pub fn bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn selected_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    pub fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    pub fn padding(&self, _state: &ListItemRenderState, theme: &Theme) -> Edges<Pixels> {
        // Horizontal uses the list_item-specific token (default 12);
        // vertical falls back to the small inset (default 4).
        Edges::symmetric(
            gpui::px(
                theme
                    .get_number("tokens.control.list_item.horizontal_padding")
                    .unwrap_or_else(|| theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0))
                    as f32,
            ),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32),
        )
    }
    pub fn min_height(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.list_item.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
}

impl ListItemRenderer for WinUIListItemRenderer {
    fn compose(&self, props: &ListItemProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ListItemRenderState {
            selected: props.selected,
            disabled: props.disabled,
            hovered: false,
        };
        let bg = if state.selected {
            self.selected_bg(&state, theme)
        } else {
            self.bg(&state, theme)
        };
        let hover_bg = self.hover_bg(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let clickable = props.on_click.is_some() && !props.disabled;

        let mut el: Div = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .bg(bg)
            .text_color(fg)
            .px(pad.left)
            .py(pad.top)
            .min_h(h)
            .rounded(r)
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else if clickable {
                CursorStyle::PointingHand
            } else {
                CursorStyle::Arrow
            });

        if let Some(lead) = &props.leading_icon {
            let id = format!("{:?}-leading", props.id).into();
            el = el.child(
                IconProps {
                    id,
                    source: IconSource::Builtin(lead.clone()),
                    size: Some(px(16.0)),
                    color: Some(fg),
                }
                .render(cx),
            );
        }
        el = el.child(props.title.to_string());
        if let Some(trail) = &props.trailing_icon {
            let id = format!("{:?}-trailing", props.id).into();
            el = el.child(
                IconProps {
                    id,
                    source: IconSource::Builtin(trail.clone()),
                    size: Some(px(14.0)),
                    color: Some(fg),
                }
                .render(cx),
            );
        }
        if clickable {
            el = el.hover(move |s| s.bg(hover_bg));
        }
        el
    }
}

pub fn arc_list_item<T: ListItemRenderer + 'static>(r: T) -> Arc<dyn ListItemRenderer> {
    Arc::new(r)
}
