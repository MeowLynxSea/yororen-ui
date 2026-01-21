use gpui::{
    AbsoluteLength, DefiniteLength, Div, IntoElement, ParentElement, RenderOnce, Styled, div,
    prelude::FluentBuilder,
};

pub fn button_group() -> ButtonGroup {
    ButtonGroup::new()
}

#[derive(IntoElement)]
pub struct ButtonGroup {
    base: Div,
    children: Vec<gpui::AnyElement>,
    gap: Option<DefiniteLength>,
    radius: Option<AbsoluteLength>,
    connected: bool,
}

impl Default for ButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonGroup {
    pub fn new() -> Self {
        Self {
            base: div(),
            children: Vec::new(),
            gap: None,
            radius: None,
            connected: false,
        }
    }

    pub fn gap(mut self, gap: DefiniteLength) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn radius(mut self, radius: AbsoluteLength) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }
}

impl ParentElement for ButtonGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for ButtonGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let gap = self.gap;
        let radius = self.radius;
        let connected = self.connected;

        let mut group = self.base.flex().items_center();
        if let Some(gap) = gap
            && !connected
        {
            group = group.gap(gap);
        }

        if connected {
            group = group
                .when_some(radius, |this, radius| this.rounded(radius))
                .overflow_hidden();
        }

        group.children(self.children)
    }
}
