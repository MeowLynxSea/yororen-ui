use std::sync::Arc;

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::component::{compute_action_style, ClickCallback, HoverCallback};
use crate::theme::{ActionVariantKind, ActiveTheme};

/// Creates a new button element.
///
/// Buttons trigger actions or navigation. Use `.variant()` to change visual style
/// (e.g., `ActionVariantKind::Primary`, `ActionVariantKind::Danger`).
///
/// # Example
/// ```rust
/// use yororen_ui::component::{button, ActionVariantKind};
///
/// let btn = button("my-button")
///     .variant(ActionVariantKind::Primary)
///     .child("Click me")
///     .on_click(|_ev, _window, _cx| {
///         // handle click
///     });
/// ```
///
/// Note: The first argument must be a unique, stable element ID.
pub fn button(id: impl Into<ElementId>) -> Button {
    Button::new().id(id)
}

#[derive(IntoElement)]
pub struct Button {
    element_id: ElementId,
    base: Div,

    click_fn: Option<ClickCallback>,
    hover_fn: Option<HoverCallback>,
    clickable: bool,
    disabled: bool,
    variant: ActionVariantKind,

    bg: Option<Hsla>,
    hover_bg: Option<Hsla>,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    /// Creates a new button with default styles.
    ///
    /// Default height is 36px with horizontal and vertical padding.
    /// Default variant is `ActionVariantKind::Neutral`.
    ///
    /// # Example
    /// ```rust
    /// button()
    ///     .id("my-button")
    ///     .on_click(|_ev, _window, _cx| { /* ... */ })
    /// ```
    ///
    /// Note: Always provide a unique, stable ID via `.id()` for interactive components.
    pub fn new() -> Self {
        Self {
            element_id: "button".into(),
            base: div().h(px(36.)).px_4().py_2(),
            click_fn: None,
            hover_fn: None,
            clickable: true,
            disabled: false,
            variant: ActionVariantKind::Neutral,
            bg: None,
            hover_bg: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
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
        self.click_fn = Some(Arc::new(listener));
        self
    }

    pub fn on_hover<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(bool, &mut gpui::Window, &mut gpui::App),
    {
        self.hover_fn = Some(Arc::new(listener));
        self
    }

    pub fn bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.bg = Some(fill.into());
        self
    }

    pub fn hover_bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.hover_bg = Some(fill.into());
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
        let bg = self.bg;
        let hover_bg = self.hover_bg;
        let variant = self.variant;

        let action_style = compute_action_style(cx.theme(), variant, disabled, bg, hover_bg);

        self.base.id(self.element_id)
            .rounded_md()
            .flex()
            .items_center()
            .justify_center()
            .bg(action_style.bg)
            .text_color(action_style.fg)
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
            .hover(move |this| this.bg(action_style.hover_bg))
    }
}
