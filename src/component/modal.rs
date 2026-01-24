use gpui::prelude::FluentBuilder;
use gpui::{
    Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, div, px,
};

use crate::{
    component::{HeadingLevel, button, heading, label},
    theme::{ActionVariantKind, ActiveTheme},
};

/// Modal content shell (dialog panel).
///
/// This component only renders the *panel* (title/content/actions slots) and is
/// intentionally not responsible for overlay / focus trapping.
///
/// Use it inside a popover/overlay layer in your app.
pub fn modal() -> Modal {
    Modal::new()
}

#[derive(IntoElement)]
pub struct Modal {
    base: gpui::Div,
    title: Option<SharedString>,
    content: Option<gpui::AnyElement>,
    actions: Option<gpui::AnyElement>,
    width: gpui::Pixels,
    bg: Option<Hsla>,
    border: Option<Hsla>,
}

impl Default for Modal {
    fn default() -> Self {
        Self::new()
    }
}

impl Modal {
    pub fn new() -> Self {
        Self {
            base: div(),
            title: None,
            content: None,
            actions: None,
            width: px(520.),
            bg: None,
            border: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border = Some(color.into());
        self
    }
}

impl ParentElement for Modal {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Modal {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Modal {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let bg = self.bg.unwrap_or(theme.surface.raised);
        let border = self.border.unwrap_or(theme.border.default);

        let title = self.title;
        let has_title = title.is_some();
        let content = self
            .content
            .unwrap_or_else(|| label("Content").muted(true).into_any_element());
        let actions = self.actions;

        self.base
            .id("ui:modal")
            .w(self.width)
            .rounded_lg()
            .border_1()
            .border_color(border)
            .bg(bg)
            .shadow_md()
            .overflow_hidden()
            .child(
                div()
                    .px_4()
                    .py_3()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .when_some(title, |this, title| {
                                this.child(heading(title).level(HeadingLevel::H3))
                            })
                            .when(!has_title, |this| this.child(label("Modal").muted(true))),
                    ),
            )
            .child(div().h(px(1.)).w_full().bg(theme.border.divider))
            .child(div().px_4().py_4().child(content))
            .when_some(actions, |this, actions| {
                this.child(div().h(px(1.)).w_full().bg(theme.border.divider))
                    .child(div().px_4().py_3().child(actions))
            })
    }
}

pub fn modal_actions_row(children: impl IntoIterator<Item = gpui::AnyElement>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_end()
        .gap_2()
        .children(children)
}

pub fn modal_primary_action(label_text: impl Into<SharedString>) -> impl IntoElement {
    button()
        .variant(ActionVariantKind::Primary)
        .child(label_text.into())
}
