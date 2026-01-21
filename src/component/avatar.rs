use std::sync::Arc;

use gpui::{
    Div, Hsla, Image, IntoElement, ObjectFit, ParentElement, RenderOnce, Styled, StyledImage, div,
    img, prelude::FluentBuilder, px,
};

use crate::theme::ActiveTheme;

pub fn avatar(image: Option<Arc<Image>>) -> Avatar {
    Avatar::new(image)
}

#[derive(Clone, Copy)]
pub enum AvatarShape {
    Circle,
    Square,
}

#[derive(IntoElement)]
pub struct Avatar {
    base: Div,
    image: Option<Arc<Image>>,
    shape: AvatarShape,
    bg_color: Option<Hsla>,
    status: Option<Hsla>,
}

impl Avatar {
    pub fn new(image: Option<Arc<Image>>) -> Self {
        Self {
            base: div(),
            image,
            shape: AvatarShape::Circle,
            bg_color: None,
            status: None,
        }
    }

    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg_color = Some(color.into());
        self
    }

    pub fn status(mut self, color: impl Into<Hsla>) -> Self {
        self.status = Some(color.into());
        self
    }
}

impl ParentElement for Avatar {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Avatar {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let is_circle = matches!(self.shape, AvatarShape::Circle);

        let mut base = self
            .base
            .size_10()
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().content.tertiary);

        if let Some(bg) = self.bg_color {
            base = base.bg(bg);
        }

        base = match self.shape {
            AvatarShape::Circle => base.rounded_full(),
            AvatarShape::Square => base.rounded_md(),
        };

        let base = if let Some(image) = self.image {
            base.child(
                img(image)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .when(is_circle, |this| this.rounded_full())
                    .when(!is_circle, |this| this.rounded_md()),
            )
        } else {
            base.child("?")
        };

        base.when_some(self.status, |this, color| {
            this.child(
                div()
                    .absolute()
                    .right(px(2.))
                    .bottom(px(2.))
                    .size_3()
                    .rounded_full()
                    .bg(color)
                    .border_2()
                    .border_color(cx.theme().surface.base),
            )
        })
    }
}
