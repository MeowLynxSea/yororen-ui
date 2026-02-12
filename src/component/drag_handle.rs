use gpui::{
    Div, ElementId, Hsla, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, ParentElement, RenderOnce, Styled, div, prelude::FluentBuilder,
};

use crate::theme::{ActionVariantKind, ActiveTheme};

/// Creates a new drag handle element.
pub fn drag_handle() -> DragHandle {
    DragHandle::new()
}

type DragMoveFn = Box<dyn Fn(&MouseMoveEvent, &mut gpui::Window, &mut gpui::App)>;

type DragStartFn = Box<dyn Fn(&MouseDownEvent, &mut gpui::Window, &mut gpui::App)>;

type DragEndFn = Box<dyn Fn(&MouseMoveEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct DragHandle {
    element_id: Option<ElementId>,
    base: Div,

    on_drag_start: Option<DragStartFn>,
    on_drag_move: Option<DragMoveFn>,
    on_drag_end: Option<DragEndFn>,

    enabled: bool,
    dragging: bool,
    variant: ActionVariantKind,

    bg_color: Option<Hsla>,
    hover_bg_color: Option<Hsla>,
}

impl Default for DragHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl DragHandle {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),

            on_drag_start: None,
            on_drag_move: None,
            on_drag_end: None,

            enabled: true,
            dragging: false,
            variant: ActionVariantKind::Neutral,

            bg_color: None,
            hover_bg_color: None,
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

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn dragging(mut self, dragging: bool) -> Self {
        self.dragging = dragging;
        self
    }

    pub fn variant(mut self, variant: ActionVariantKind) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_drag_start<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&MouseDownEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_drag_start = Some(Box::new(listener));
        self
    }

    pub fn on_drag_move<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&MouseMoveEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_drag_move = Some(Box::new(listener));
        self
    }

    pub fn on_drag_end<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&MouseMoveEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_drag_end = Some(Box::new(listener));
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

impl ParentElement for DragHandle {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for DragHandle {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for DragHandle {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let enabled = self.enabled;
        let dragging = self.dragging;
        let on_drag_start = self.on_drag_start;
        let on_drag_move = self.on_drag_move;
        let on_drag_end = self.on_drag_end;
        let bg = self.bg_color;
        let hover_bg = self.hover_bg_color;
        let variant = self.variant;
        let element_id = self.element_id;

        let action_variant = _cx.theme().action_variant(variant);
        let hover_bg = hover_bg.unwrap_or(action_variant.hover_bg);
        let mut resolved_bg = bg.unwrap_or(action_variant.bg);
        let mut resolved_hover_bg = hover_bg;

        if !enabled {
            resolved_bg = action_variant.disabled_bg;
            resolved_hover_bg = action_variant.disabled_bg;
        }

        self.base.id(element_id.unwrap_or_else(|| "".into()))
            .when(enabled, |this| this.cursor_grab())
            .when(dragging, |this| this.cursor_grabbing())
            .bg(resolved_bg)
            .hover(move |this| this.bg(resolved_hover_bg))
            .on_mouse_down(MouseButton::Left, move |ev, window, cx| {
                if enabled && let Some(handler) = &on_drag_start {
                    handler(ev, window, cx);
                }
            })
            .on_mouse_move(move |ev, window, cx| {
                if enabled && ev.dragging() {
                    if let Some(handler) = &on_drag_move {
                        handler(ev, window, cx);
                    }
                } else if enabled && let Some(handler) = &on_drag_end {
                    handler(ev, window, cx);
                }
            })
    }
}
