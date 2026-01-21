use gpui::{Div, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div, px};

use crate::theme::ActiveTheme;

pub fn divider() -> Divider {
    Divider::new()
}

#[derive(IntoElement)]
pub struct Divider {
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
            base: div(),
            vertical: false,
        }
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
        if self.vertical {
            self.base
                .id("divider-vertical")
                .w(px(1.))
                .h_full()
                .bg(cx.theme().border.divider)
        } else {
            self.base
                .id("divider-horizontal")
                .h(px(1.))
                .w_full()
                .bg(cx.theme().border.divider)
        }
    }
}
