use gpui::{
    Div, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, div, prelude::FluentBuilder,
};

use crate::theme::ActiveTheme;

pub fn label(text: impl Into<SharedString>) -> Label {
    Label::new(text)
}

#[derive(IntoElement)]
pub struct Label {
    element_id: Option<ElementId>,
    base: Div,
    text: SharedString,

    muted: bool,
    strong: bool,
    inherit_color: bool,
    mono: bool,
    ellipsis: bool,
    max_lines: Option<usize>,

    preview_lines: Option<usize>,
}

impl Label {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            element_id: None,
            base: div(),
            text: text.into(),

            muted: false,
            strong: false,
            inherit_color: false,
            mono: false,
            ellipsis: false,
            max_lines: None,

            preview_lines: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    pub fn muted(mut self, value: bool) -> Self {
        self.muted = value;
        self
    }

    pub fn strong(mut self, value: bool) -> Self {
        self.strong = value;
        self
    }

    pub fn inherit_color(mut self, value: bool) -> Self {
        self.inherit_color = value;
        self
    }

    pub fn mono(mut self, value: bool) -> Self {
        self.mono = value;
        self
    }

    pub fn ellipsis(mut self, value: bool) -> Self {
        self.ellipsis = value;
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }

    /// Render a multi-line preview that clamps to `lines`.
    ///
    /// This is designed for previews of potentially multi-paragraph content (news, descriptions).
    /// It will:
    ///
    /// - Keep only the first paragraph (split on a blank line).
    /// - Trim trailing whitespace.
    /// - Apply line clamping to `lines`.
    ///
    /// Use the original full text in a modal/popover when the user clicks "read more".
    pub fn preview_lines(mut self, lines: usize) -> Self {
        self.preview_lines = Some(lines);
        self
    }
}

impl ParentElement for Label {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Label {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = if let Some(id) = self.element_id {
            id
        } else {
            self.text.clone().into()
        };

        let mut base = self
            .base
            .id(id)
            .when(self.strong, |this| this.font_weight(FontWeight::SEMIBOLD))
            .when(self.mono, |this| this.font_family("monospace"))
            .when(self.ellipsis, |this| this.truncate())

            // If both are provided, `preview_lines` wins: it also controls the line clamp.
            .when_some(self.preview_lines, |this, lines| this.relative().line_clamp(lines))
            .when(self.preview_lines.is_none(), |this| {
                this.when_some(self.max_lines, |this, lines| this.line_clamp(lines))
            });

        if let Some(_lines) = self.preview_lines {
            let full = self.text.as_ref();
            let mut paragraphs = full.split("\n\n");
            let first_paragraph = paragraphs.next().unwrap_or("");

            let trimmed = first_paragraph.trim_end();

            let preview_text: SharedString = if trimmed.is_empty() {
                self.text
            } else {
                // Prevent previews from accidentally showing the next paragraph.
                SharedString::from(trimmed.replace('\n', " "))
            };

            base = base.child(preview_text);
        } else {
            base = base.child(self.text);
        }

        if self.inherit_color {
            base
        } else {
            base.text_color(if self.muted {
                cx.theme().content.secondary
            } else {
                cx.theme().content.primary
            })
        }
    }
}
