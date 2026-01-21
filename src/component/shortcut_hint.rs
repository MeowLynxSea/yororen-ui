use gpui::{Div, Hsla, IntoElement, ParentElement, RenderOnce, Styled, div};

use crate::theme::ActiveTheme;

pub fn shortcut_hint(hint: impl Into<String>) -> ShortcutHint {
    ShortcutHint::new(hint)
}

#[derive(IntoElement)]
pub struct ShortcutHint {
    base: Div,
    hint: String,
    tone: Option<Hsla>,
}

impl ShortcutHint {
    pub fn new(hint: impl Into<String>) -> Self {
        Self {
            base: div(),
            hint: hint.into(),
            tone: None,
        }
    }

    pub fn tone(mut self, color: impl Into<Hsla>) -> Self {
        self.tone = Some(color.into());
        self
    }
}

impl ParentElement for ShortcutHint {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ShortcutHint {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ShortcutHint {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let bg = self.tone.unwrap_or_else(|| cx.theme().surface.sunken);
        self.base
            .px_2()
            .py_1()
            .rounded_sm()
            .bg(bg)
            .text_xs()
            .text_color(cx.theme().content.tertiary)
            .child(self.hint)
    }
}
