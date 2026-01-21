use std::panic::Location;

use gpui::{
    Div, ElementId, Hsla, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, RenderOnce, Styled, div, prelude::FluentBuilder,
};

use crate::theme::{ActionVariantKind, ActiveTheme};

#[track_caller]
pub fn context_menu_trigger() -> ContextMenuTrigger {
    ContextMenuTrigger::new().id(ElementId::from(Location::caller()))
}

type OpenFn = Box<dyn Fn(&MouseDownEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct ContextMenuTrigger {
    element_id: Option<ElementId>,
    base: Div,

    on_open: Option<OpenFn>,
    consume: bool,
    enabled: bool,
    variant: ActionVariantKind,

    bg_color: Option<Hsla>,
    hover_bg_color: Option<Hsla>,
}

impl Default for ContextMenuTrigger {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenuTrigger {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            element_id: Some(ElementId::from(Location::caller())),
            base: div(),

            on_open: None,
            consume: true,
            enabled: true,
            variant: ActionVariantKind::Neutral,

            bg_color: None,
            hover_bg_color: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn consume(mut self, consume: bool) -> Self {
        self.consume = consume;
        self
    }

    pub fn variant(mut self, variant: ActionVariantKind) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_open<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&MouseDownEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_open = Some(Box::new(listener));
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

impl ParentElement for ContextMenuTrigger {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ContextMenuTrigger {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for ContextMenuTrigger {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl RenderOnce for ContextMenuTrigger {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let enabled = self.enabled;
        let consume = self.consume;
        let on_open = self.on_open;
        let bg = self.bg_color;
        let hover_bg = self.hover_bg_color;
        let variant = self.variant;

        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let action_variant = _cx.theme().action_variant(variant);
        let hover_bg = hover_bg.unwrap_or(action_variant.hover_bg);
        let mut resolved_bg = bg.unwrap_or(action_variant.bg);

        if !enabled {
            resolved_bg = action_variant.disabled_bg;
        }

        self.base
            .id(id)
            .when(enabled, |this| this.cursor_context_menu())
            .on_mouse_down(MouseButton::Right, move |ev, window, cx| {
                if !enabled {
                    return;
                }

                if consume {
                    cx.stop_propagation();
                }

                if let Some(handler) = &on_open {
                    handler(ev, window, cx);
                }
            })
            .bg(resolved_bg)
            .hover(move |this| this.bg(hover_bg))
    }
}
