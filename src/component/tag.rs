use gpui::{
    ClickEvent, Div, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, div, px,
};

use crate::{
    component::{IconName, icon},
    theme::ActiveTheme,
};

type OnCloseHandler = dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App);

pub fn tag(text: impl Into<String>) -> Tag {
    Tag::new(text)
}

#[derive(IntoElement)]
pub struct Tag {
    base: Div,
    text: String,
    selected: bool,
    closable: bool,
    on_close: Option<Box<OnCloseHandler>>,
    tone: Option<Hsla>,
}

impl Tag {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            base: div(),
            text: text.into(),
            selected: false,
            closable: false,
            on_close: None,
            tone: None,
        }
    }

    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    pub fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }

    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_close = Some(Box::new(handler));
        self
    }

    pub fn tone(mut self, color: impl Into<Hsla>) -> Self {
        self.tone = Some(color.into());
        self
    }
}

impl ParentElement for Tag {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Tag {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Tag {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let bg = self.tone.unwrap_or_else(|| cx.theme().action.neutral.bg);
        let tone_fg = if self.tone.is_some() {
            cx.theme().content.on_status
        } else {
            cx.theme().action.neutral.fg
        };

        let mut base = self
            .base
            .h(px(26.))
            .px_2()
            .rounded_full()
            .bg(if self.selected {
                cx.theme().action.primary.bg
            } else {
                bg
            })
            .text_color(if self.selected {
                cx.theme().action.primary.fg
            } else {
                tone_fg
            })
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .flex()
            .items_center()
            .gap_1()
            .child(self.text);

        if self.closable {
            let on_close = self.on_close;
            base = base.child(
                div()
                    .id("tag-close")
                    .w_4()
                    .h_4()
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|this| this.bg(cx.theme().action.neutral.hover_bg))
                    .cursor_pointer()
                    .child(icon(IconName::Close).size(px(10.)).inherit_color(true))
                    .on_click(move |ev, window, cx| {
                        if let Some(handler) = &on_close {
                            handler(ev, window, cx);
                        }
                    }),
            );
        }

        base
    }
}
