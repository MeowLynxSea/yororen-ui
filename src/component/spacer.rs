use gpui::{Div, IntoElement, ParentElement, RenderOnce, Styled, div};

pub fn spacer() -> Spacer {
    Spacer::new()
}

#[derive(IntoElement)]
pub struct Spacer {
    base: Div,
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl Spacer {
    pub fn new() -> Self {
        Self { base: div() }
    }
}

impl ParentElement for Spacer {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Spacer {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Spacer {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.base.flex_grow()
    }
}
