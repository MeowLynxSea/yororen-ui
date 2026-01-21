use std::panic::Location;

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActionVariantKind, ActiveTheme};

#[track_caller]
pub fn button() -> Button {
    Button::new().id(ElementId::from(Location::caller()))
}

type ClickFn = Box<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App)>;

type HoverFn = Box<dyn Fn(bool, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct Button {
    element_id: Option<ElementId>,
    base: Div,

    click_fn: Option<ClickFn>,
    hover_fn: Option<HoverFn>,
    clickable: bool,
    disabled: bool,
    variant: ActionVariantKind,

    bg_color: Option<Hsla>,
    hover_bg_color: Option<Hsla>,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            element_id: Some(ElementId::from(Location::caller())),
            base: div().h(px(36.)).px_4().py_2(),
            click_fn: None,
            hover_fn: None,
            clickable: true,
            disabled: false,
            variant: ActionVariantKind::Neutral,
            bg_color: None,
            hover_bg_color: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
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
}

impl Styled for Button {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Button {}

impl RenderOnce for Button {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let clickable = self.clickable;
        let disabled = self.disabled;
        let click_fn = self.click_fn;
        let hover_fn = self.hover_fn;
        let bg = self.bg_color;
        let hover_bg = self.hover_bg_color;
        let variant = self.variant;

        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let action_variant = cx.theme().action_variant(variant);
        let mut resolved_bg = bg.unwrap_or(action_variant.bg);
        let mut resolved_hover_bg = hover_bg.unwrap_or(action_variant.hover_bg);
        let mut resolved_text_color = action_variant.fg;

        if disabled {
            resolved_bg = action_variant.disabled_bg;
            resolved_hover_bg = action_variant.disabled_bg;
            resolved_text_color = action_variant.disabled_fg;
        }

        self.base
            .id(id.clone())
            .flex()
            .items_center()
            .justify_center()
            .bg(resolved_bg)
            .text_color(resolved_text_color)
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
            .hover(move |this| this.bg(resolved_hover_bg))
    }
}
