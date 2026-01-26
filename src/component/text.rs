use gpui::{
    Div, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::component::Icon;

pub fn text(text: impl Into<SharedString>) -> Text {
    Text::new(text)
}

#[derive(IntoElement)]
pub struct Text {
    element_id: Option<ElementId>,
    base: Div,
    text: SharedString,
    icon: Option<Icon>,
}

impl Text {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            element_id: None,
            base: div(),
            text: text.into(),
            icon: None,
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

    pub fn with_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

impl ParentElement for Text {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Text {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Text {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Text {}

impl RenderOnce for Text {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        self.base
            .id(if let Some(id) = self.element_id {
                id
            } else {
                self.text.clone().into()
            })
            .flex()
            .items_center()
            .gap_2()
            .when(self.icon.is_some(), |this| this.child(self.icon.unwrap()))
            .child(self.text)
    }
}
