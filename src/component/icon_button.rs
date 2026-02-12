use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    component::Icon,
    theme::{ActionVariantKind, ActiveTheme},
};

/// Creates a new icon button element.
pub fn icon_button(icon: impl Into<Icon>) -> IconButton {
    IconButton::new(icon)
}

type ClickFn = Box<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App)>;

type HoverFn = Box<dyn Fn(bool, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct IconButton {
    element_id: Option<ElementId>,
    base: Div,
    icon: Icon,

    click_fn: Option<ClickFn>,
    hover_fn: Option<HoverFn>,
    clickable: bool,
    disabled: bool,
    variant: ActionVariantKind,

    bg_color: Option<Hsla>,
    hover_bg_color: Option<Hsla>,
    icon_size: Option<Pixels>,
}

impl IconButton {
    pub fn new(icon: impl Into<Icon>) -> Self {
        Self {
            element_id: None,
            base: div().w(px(36.)).h(px(36.)),
            icon: icon.into(),

            click_fn: None,
            hover_fn: None,
            clickable: true,
            disabled: false,
            variant: ActionVariantKind::Neutral,

            bg_color: None,
            hover_bg_color: None,
            icon_size: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ActionVariantKind) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.click_fn = Some(Box::new(listener));
        self
    }

    pub fn on_hover<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(bool, &mut gpui::Window, &mut gpui::App),
    {
        self.hover_fn = Some(Box::new(listener));
        self
    }

    pub fn bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.bg_color = Some(fill.into());
        self
    }

    pub fn hover_bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.hover_bg_color = Some(fill.into());
        self
    }

    pub fn icon_size(mut self, size: Pixels) -> Self {
        self.icon_size = Some(size);
        self
    }
}

impl ParentElement for IconButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for IconButton {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for IconButton {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for IconButton {}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let clickable = self.clickable;
        let click_fn = self.click_fn;
        let hover_fn = self.hover_fn;
        let bg = self.bg_color;
        let hover_bg = self.hover_bg_color;
        let disabled = self.disabled;
        let variant = self.variant;
        let icon_size = self.icon_size;
        let element_id = self.element_id;

        let action_variant = cx.theme().action_variant(variant);
        let mut resolved_bg = bg.unwrap_or(action_variant.bg);
        let mut resolved_hover_bg = hover_bg.unwrap_or(action_variant.hover_bg);
        let mut resolved_text_color = action_variant.fg;

        if disabled {
            resolved_bg = action_variant.disabled_bg;
            resolved_hover_bg = resolved_bg;
            resolved_text_color = action_variant.disabled_fg;
        }

        self.base.id(element_id.unwrap_or_else(|| "".into()))
            .rounded_md()
            .flex()
            .items_center()
            .justify_center()
            .text_color(resolved_text_color)
            .focusable()
            .focus_visible(|style| style.border_2().border_color(cx.theme().border.focus))
            .when(clickable && !disabled, |this| this.cursor_pointer())
            .when(disabled, |this| this.cursor_not_allowed())
            .on_click(move |ev, window, cx| {
                if disabled {
                    return;
                }
                if clickable && let Some(f) = &click_fn {
                    f(ev, window, cx);
                }
            })
            .when(hover_fn.is_some(), move |this| {
                let hover_fn = hover_fn;
                this.on_hover(move |active, window, cx| {
                    if let Some(handler) = &hover_fn {
                        handler(*active, window, cx);
                    }
                })
            })
            .bg(resolved_bg)
            .hover(move |this| this.bg(resolved_hover_bg))
            .child(
                self.icon
                    .size(icon_size.unwrap_or(px(14.)))
                    .color(resolved_text_color),
            )
    }
}
