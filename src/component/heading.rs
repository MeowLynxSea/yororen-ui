use gpui::{
    Div, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, div, px,
};

use crate::theme::ActiveTheme;

pub fn heading(text: impl Into<SharedString>) -> Heading {
    Heading::new(text)
}

#[derive(Clone, Copy)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
}

#[derive(IntoElement)]
pub struct Heading {
    element_id: Option<ElementId>,
    base: Div,
    text: SharedString,
    level: HeadingLevel,
}

impl Heading {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            element_id: None,
            base: div(),
            text: text.into(),
            level: HeadingLevel::H2,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    pub fn level(mut self, level: HeadingLevel) -> Self {
        self.level = level;
        self
    }
}

impl ParentElement for Heading {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Heading {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Heading {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = if let Some(id) = self.element_id {
            id
        } else {
            self.text.clone().into()
        };

        let (size, weight) = match self.level {
            HeadingLevel::H1 => (32., FontWeight::BOLD),
            HeadingLevel::H2 => (24., FontWeight::SEMIBOLD),
            HeadingLevel::H3 => (18., FontWeight::SEMIBOLD),
        };

        self.base
            .id(id)
            .text_size(px(size))
            .font_weight(weight)
            .text_color(cx.theme().content.primary)
            .child(self.text)
    }
}
