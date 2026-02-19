use gpui::{
    AbsoluteLength, DefiniteLength, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Styled, div, prelude::FluentBuilder,
};

pub fn button_group() -> ButtonGroup {
    ButtonGroup::new()
}

#[derive(IntoElement)]
pub struct ButtonGroup {
    element_id: ElementId,
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
            element_id: "ui:button-group".into(),
            base: div(),
            children: Vec::new(),
            gap: None,
            radius: None,
            connected: false,
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
        let element_id = self.element_id;

        let mut group = self.base.id(element_id).flex().items_center();
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
