use std::panic::Location;

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::theme::{ActionVariantKind, ActiveTheme};

#[track_caller]
pub fn clickable_surface() -> ClickableSurface {
    ClickableSurface::new().id(ElementId::from(Location::caller()))
}

type ClickFn = Box<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App)>;

type HoverFn = Box<dyn Fn(bool, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct ClickableSurface {
    element_id: Option<ElementId>,
    base: Div,

    click_fn: Option<ClickFn>,
    hover_fn: Option<HoverFn>,

    clickable: bool,
    focusable: bool,
    variant: ActionVariantKind,

    bg_color: Option<Hsla>,
    hover_bg_color: Option<Hsla>,
    focus_ring: Option<Hsla>,
}

impl Default for ClickableSurface {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickableSurface {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            element_id: Some(ElementId::from(Location::caller())),
            base: div(),

            click_fn: None,
            hover_fn: None,

            clickable: true,
            focusable: false,
            variant: ActionVariantKind::Neutral,

            bg_color: None,
            hover_bg_color: None,
            focus_ring: None,
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

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
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

    pub fn focus_ring(mut self, color: impl Into<Hsla>) -> Self {
        self.focus_ring = Some(color.into());
        self
    }
}

impl ParentElement for ClickableSurface {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ClickableSurface {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for ClickableSurface {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for ClickableSurface {}

impl RenderOnce for ClickableSurface {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let clickable = self.clickable;
        let focusable = self.focusable;
        let click_fn = self.click_fn;
        let hover_fn = self.hover_fn;
        let bg = self.bg_color;
        let hover_bg = self.hover_bg_color;
        let focus_ring = self.focus_ring;
        let variant = self.variant;

        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let action_variant = _cx.theme().action_variant(variant);
        let hover_bg = hover_bg.unwrap_or(action_variant.hover_bg);
        let focus_ring = focus_ring.unwrap_or(_cx.theme().border.focus);

        self.base
            .id(id)
            .when(focusable, |this| this.focusable())
            .when(clickable, |this| this.cursor_pointer())
            .on_click(move |ev, window, cx| {
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
            .bg(bg.unwrap_or(action_variant.bg))
            .hover(move |this| this.bg(hover_bg))
            .focus_visible(|style| style.border_2().border_color(focus_ring))
    }
}
