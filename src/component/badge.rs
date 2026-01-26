use std::panic::Location;

use gpui::{
    Div, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, div,
};

use crate::theme::ActiveTheme;

#[track_caller]
pub fn badge(text: impl Into<SharedString>) -> Badge {
    Badge::new(text)
}

#[derive(IntoElement)]
pub struct Badge {
    element_id: Option<ElementId>,
    base: Div,
    text: SharedString,
    tone: Option<Hsla>,
}

impl Badge {
    #[track_caller]
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            element_id: Some(ElementId::from(Location::caller())),
            base: div(),
            text: text.into(),
            tone: None,
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

    pub fn tone(mut self, color: impl Into<Hsla>) -> Self {
        self.tone = Some(color.into());
        self
    }
}

impl ParentElement for Badge {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Badge {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let default_bg = cx.theme().status.info.bg;
        let bg = self.tone.unwrap_or(default_bg);
        let fg = if self.tone.is_some() {
            cx.theme().content.on_status
        } else {
            cx.theme().status.info.fg
        };

        self.base
            .id(id)
            .px_2()
            .h_5()
            .rounded_full()
            .bg(bg)
            .text_color(fg)
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .flex()
            .items_center()
            .child(self.text)
    }
}
