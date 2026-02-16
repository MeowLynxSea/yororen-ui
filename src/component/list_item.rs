use gpui::{
    Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div,
    prelude::FluentBuilder, px,
};

use crate::theme::ActiveTheme;

/// A row content container for list-style UIs.
///
/// Responsibilities:
/// - standard layout slots: leading / content / secondary / trailing
/// - hover + selected background
///
/// Note: virtualization shell responsibilities (spacing/dividers/stable row ids)
/// are handled by [`crate::component::VirtualRow`].
pub fn list_item() -> ListItem {
    ListItem::new()
}

#[derive(IntoElement)]
pub struct ListItem {
    element_id: ElementId,
    base: Div,
    leading: Option<gpui::AnyElement>,
    content: Option<gpui::AnyElement>,
    secondary: Option<gpui::AnyElement>,
    trailing: Option<gpui::AnyElement>,
    hoverable: bool,
    selected: bool,
    hover_bg: Option<Hsla>,
    selected_bg: Option<Hsla>,
}

impl Default for ListItem {
    fn default() -> Self {
        Self::new()
    }
}

impl ListItem {
    pub fn new() -> Self {
        Self {
            element_id: "ui:list-item".into(),
            base: div(),
            leading: None,
            content: None,
            secondary: None,
            trailing: None,
            hoverable: true,
            selected: false,
            hover_bg: None,
            selected_bg: None,
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

    pub fn leading(mut self, el: impl IntoElement) -> Self {
        self.leading = Some(el.into_any_element());
        self
    }

    pub fn content(mut self, el: impl IntoElement) -> Self {
        self.content = Some(el.into_any_element());
        self
    }

    pub fn secondary(mut self, el: impl IntoElement) -> Self {
        self.secondary = Some(el.into_any_element());
        self
    }

    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        self.trailing = Some(el.into_any_element());
        self
    }

    pub fn hoverable(mut self, hoverable: bool) -> Self {
        self.hoverable = hoverable;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn hover_bg(mut self, bg: impl Into<Hsla>) -> Self {
        self.hover_bg = Some(bg.into());
        self
    }

    pub fn selected_bg(mut self, bg: impl Into<Hsla>) -> Self {
        self.selected_bg = Some(bg.into());
        self
    }
}

impl ParentElement for ListItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ListItem {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ListItem {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let hoverable = self.hoverable;
        let selected = self.selected;
        let hover_bg = self.hover_bg.unwrap_or(cx.theme().surface.hover);
        let selected_bg = self
            .selected_bg
            .unwrap_or(cx.theme().action.neutral.active_bg);

        let leading = self.leading;
        let content = self.content;
        let secondary = self.secondary;
        let trailing = self.trailing;

        self.base.id(self.element_id)
            .w_full()
            .min_h(px(32.))
            .px_3()
            .py_2()
            .rounded_md()
            .flex()
            .items_center()
            .gap_2()
            .when(selected, move |this| this.bg(selected_bg))
            .when(hoverable && !selected, move |this| {
                this.hover(|this| this.bg(hover_bg))
            })
            .children(leading)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .items_start()
                    .flex_grow()
                    .children(content)
                    .children(secondary.map(|el| {
                        div()
                            .text_sm()
                            .text_color(cx.theme().content.secondary)
                            .child(el)
                    })),
            )
            .children(trailing)
    }
}
