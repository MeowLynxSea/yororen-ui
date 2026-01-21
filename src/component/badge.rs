use gpui::{Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Styled, div};

use crate::theme::ActiveTheme;

pub fn badge(text: impl Into<String>) -> Badge {
    Badge::new(text)
}

#[derive(IntoElement)]
pub struct Badge {
    base: Div,
    text: String,
    tone: Option<Hsla>,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            base: div(),
            text: text.into(),
            tone: None,
        }
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
        let default_bg = cx.theme().status.info.bg;
        let bg = self.tone.unwrap_or(default_bg);
        let fg = if self.tone.is_some() {
            cx.theme().content.on_status
        } else {
            cx.theme().status.info.fg
        };

        self.base
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
