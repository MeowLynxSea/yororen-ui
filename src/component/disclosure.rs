use std::panic::Location;

use gpui::{
    ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div, px,
};

use crate::component::{ArrowDirection, IconName, icon};
use crate::theme::ActiveTheme;

/// A disclosure arrow with expanded/collapsed state.
///
/// This is a visual primitive only. It does not manage state by itself.
#[track_caller]
pub fn disclosure() -> Disclosure {
    Disclosure::new().id(ElementId::from(Location::caller()))
}

#[derive(IntoElement)]
pub struct Disclosure {
    element_id: Option<ElementId>,
    expanded: bool,
    size: gpui::Pixels,
}

impl Default for Disclosure {
    fn default() -> Self {
        Self::new()
    }
}

impl Disclosure {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            element_id: Some(ElementId::from(Location::caller())),
            expanded: false,
            size: px(14.),
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn size(mut self, size: gpui::Pixels) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Disclosure {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self.element_id.unwrap_or_else(|| "disclosure".into());
        let expanded = self.expanded;
        let size = self.size;

        div()
            .id(id)
            .w(size)
            .h(size)
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().content.tertiary)
            .child(
                icon(IconName::Arrow(if expanded {
                    ArrowDirection::Down
                } else {
                    ArrowDirection::Right
                }))
                .size(size),
            )
    }
}
