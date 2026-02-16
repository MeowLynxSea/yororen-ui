use gpui::{
    AbsoluteLength, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::theme::ActiveTheme;

pub fn focus_ring() -> FocusRing {
    FocusRing::new()
}

#[derive(IntoElement)]
pub struct FocusRing {
    element_id: ElementId,
    base: Div,
    color: Option<Hsla>,
    radius: Option<AbsoluteLength>,
}

impl Default for FocusRing {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusRing {
    pub fn new() -> Self {
        Self {
            element_id: "ui:focus-ring".into(),
            base: div(),
            color: None,
            radius: None,
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

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn radius(mut self, radius: AbsoluteLength) -> Self {
        self.radius = Some(radius);
        self
    }
}

impl ParentElement for FocusRing {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for FocusRing {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for FocusRing {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for FocusRing {}

impl RenderOnce for FocusRing {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let color = self.color;
        let radius = self.radius;

        let id = self.element_id.clone();

        let ring_color = color.unwrap_or_else(|| _cx.theme().border.focus);

        self.base
            .id(id)
            .focusable()
            .when_some(radius, |this, radius| this.rounded(radius))
            .focus_visible(|style| style.border_2().border_color(ring_color))
    }
}
