use gpui::{
    AnyView, AppContext, Hsla, IntoElement, ParentElement, Render, RenderOnce, Styled, div,
};

use crate::theme::ActiveTheme;

pub enum TooltipPlacement {
    Auto,
    Top,
    Right,
    Bottom,
    Left,
}

pub fn tooltip(content: impl Into<String>) -> Tooltip {
    Tooltip::text(content)
}

#[derive(IntoElement)]
pub struct Tooltip {
    content: String,
    placement: TooltipPlacement,
    bg_color: Option<Hsla>,
    text_color: Option<Hsla>,
}

struct TooltipView {
    content: String,
    bg_color: Option<Hsla>,
    text_color: Option<Hsla>,
}

impl Tooltip {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            placement: TooltipPlacement::Auto,
            bg_color: None,
            text_color: None,
        }
    }

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg_color = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    pub fn build(self) -> impl Fn(&mut gpui::Window, &mut gpui::App) -> AnyView {
        let content = self.content;
        let _placement = self.placement;
        let bg_color = self.bg_color;
        let text_color = self.text_color;
        move |_, cx| {
            cx.new(|_| TooltipView {
                content: content.clone(),
                bg_color,
                text_color,
            })
            .into()
        }
    }
}

impl Render for TooltipView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .px_3()
            .py_2()
            .rounded_sm()
            .text_sm()
            .bg(self.bg_color.unwrap_or_else(|| theme.action.neutral.bg))
            .text_color(self.text_color.unwrap_or_else(|| theme.action.neutral.fg))
            .child(self.content.clone())
    }
}

impl RenderOnce for Tooltip {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div().child(self.content)
    }
}
