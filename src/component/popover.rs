use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, ClickEvent, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Styled, div, px,
};

use crate::{animation::constants::duration, theme::ActiveTheme};

use crate::animation::ease_out_quint_clamped;

/// Defines the placement position of a popover relative to its trigger element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopoverPlacement {
    /// Positions the popover below the trigger, aligned to the start (left in LTR).
    BottomStart,
    /// Positions the popover below the trigger, aligned to the end (right in LTR).
    BottomEnd,
}

/// Creates a new popover element.
///
/// Popovers display floating content relative to a trigger element. Use `.trigger()` to set
/// the clickable element and `.content()` to set the popover body.
///
/// # Example
/// ```rust,ignore
/// use gpui::px;
/// use yororen_ui::component::{button, popover};
///
/// let p = popover()
///     .trigger(button().child("Open"))
///     .content(div().p_4().child("Popover content"))
///     .width(px(200.));
/// ```
pub fn popover(id: impl Into<ElementId>) -> Popover {
    Popover::new(id)
}

type CloseFn = Box<dyn Fn(&mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct Popover {
    element_id: ElementId,
    base: gpui::Div,

    open: bool,
    placement: PopoverPlacement,
    width: Option<gpui::Pixels>,

    trigger: Option<gpui::AnyElement>,
    content: Option<gpui::AnyElement>,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    on_close: Option<CloseFn>,
}

impl Default for Popover {
    fn default() -> Self {
        Self::new("ui:popover")
    }
}

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            element_id: id.into(),
            base: div(),

            open: false,
            placement: PopoverPlacement::BottomStart,
            width: None,

            trigger: None,
            content: None,

            bg: None,
            border: None,
            on_close: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Returns the element's ID.
    pub fn element_id(&self) -> &ElementId {
        &self.element_id
    }

    /// Generates a child element ID by combining the base element ID with a suffix.
    ///
    /// This is useful for creating unique IDs for child elements while maintaining
    /// a clear relationship to the parent component's ID.
    ///
    /// # Example
    /// ```rust,ignore
    /// let popover = popover("my-popover");
    /// let trigger_id = popover.child_id("trigger"); // "my-popover-trigger"
    /// let content_id = popover.child_id("content"); // "my-popover-content"
    /// ```
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }

    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border = Some(color.into());
        self
    }

    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn on_close<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(&mut gpui::Window, &mut gpui::App),
    {
        self.on_close = Some(Box::new(f));
        self
    }
}

impl ParentElement for Popover {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Popover {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Popover {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let element_id = self.element_id;
        let id = element_id.clone();

        let theme = cx.theme();
        let bg = self.bg.unwrap_or(theme.surface.raised);
        let border = self.border.unwrap_or(theme.border.default);

        let is_open = self.open;
        let placement = self.placement;
        let width = self.width;
        let on_close = self.on_close;

        let trigger = self.trigger.unwrap_or_else(|| div().into_any_element());
        let content = self.content.unwrap_or_else(|| div().into_any_element());

        // Like Select/ComboBox, Popover is a relative container and the menu is an absolute child
        // rendered via `gpui::deferred(...)` so it is painted above.
        self.base
            .id(element_id)
            .relative()
            .child(trigger)
            .when(is_open, move |this| {
                let menu = div()
                    .id((id.clone(), "ui:popover:menu"))
                    .absolute()
                    .when(placement == PopoverPlacement::BottomStart, |this| {
                        this.top_full().left_0()
                    })
                    .when(placement == PopoverPlacement::BottomEnd, |this| {
                        this.top_full().right_0()
                    })
                    .mt(px(10.))
                    .rounded_md()
                    .overflow_hidden()
                    .border_1()
                    .border_color(border)
                    .bg(bg)
                    .shadow_md()
                    .py_1()
                    .when_some(width, |this, width| this.w(width))
                    .occlude()
                    .on_mouse_down_out(move |_ev, window, cx| {
                        if let Some(on_close) = &on_close {
                            on_close(window, cx);
                        }
                    })
                    .child(content);

                let animated = menu.with_animation(
                    format!("ui:popover:menu:{}", is_open),
                    Animation::new(duration::MENU_OPEN).with_easing(ease_out_quint_clamped),
                    |this, value| this.opacity(value).mt(px(10.0 - 6.0 * value)),
                );

                this.child(gpui::deferred(animated).with_priority(100))
            })
    }
}

// Keep a stable signature for downstream; on_trigger click handling stays with caller.
#[allow(dead_code)]
fn _click_passthrough(_ev: &ClickEvent) {}
