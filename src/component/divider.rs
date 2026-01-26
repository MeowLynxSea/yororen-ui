use std::panic::Location;

use gpui::{
    Div, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div, px,
};

use crate::theme::ActiveTheme;

pub fn divider() -> Divider {
    Divider::new().id(ElementId::from(Location::caller()))
}

#[derive(IntoElement)]
pub struct Divider {
    element_id: Option<ElementId>,
    base: Div,
    vertical: bool,
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

impl Divider {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),
            vertical: false,
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

    pub fn vertical(mut self, value: bool) -> Self {
        self.vertical = value;
        self
    }
}

impl ParentElement for Divider {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Divider {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        if self.vertical {
            self.base
                .id(id)
                .w(px(1.))
                .h_full()
                .bg(cx.theme().border.divider)
        } else {
            self.base
                .id(id)
                .h(px(1.))
                .w_full()
                .bg(cx.theme().border.divider)
        }
    }
}
