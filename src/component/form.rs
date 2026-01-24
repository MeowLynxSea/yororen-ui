use gpui::prelude::FluentBuilder;
use gpui::{Div, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, div, px};

use crate::{
    component::{IconName, icon, label},
    theme::ActiveTheme,
};

pub fn form() -> Form {
    Form::new()
}

#[derive(IntoElement)]
pub struct Form {
    base: Div,
}

impl Form {
    pub fn new() -> Self {
        Self { base: div() }
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Form {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Form {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Form {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.base.flex().flex_col().gap_3()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationState {
    Success,
    Warning,
    Error,
}

pub fn validation_state_icon(state: ValidationState) -> ValidationStateIcon {
    ValidationStateIcon::new(state)
}

#[derive(IntoElement)]
pub struct ValidationStateIcon {
    base: Div,
    state: ValidationState,
    size: gpui::Pixels,
}

impl ValidationStateIcon {
    pub fn new(state: ValidationState) -> Self {
        Self {
            base: div(),
            state,
            size: px(22.),
        }
    }

    pub fn size(mut self, size: gpui::Pixels) -> Self {
        self.size = size;
        self
    }
}

impl Styled for ValidationStateIcon {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ValidationStateIcon {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let (icon_name, bg) = match self.state {
            ValidationState::Success => (IconName::Check, cx.theme().status.success.bg),
            ValidationState::Warning => (IconName::Warning, cx.theme().status.warning.bg),
            ValidationState::Error => (IconName::Close, cx.theme().status.error.bg),
        };

        let fg = cx.theme().content.on_status;

        self.base
            .flex()
            .items_center()
            .justify_center()
            .size(self.size)
            .rounded_sm()
            .bg(bg)
            .child(icon(icon_name).size(px(14.)).color(fg))
    }
}

pub fn help_text(text: impl Into<SharedString>) -> HelpText {
    HelpText::new(text)
}

#[derive(IntoElement)]
pub struct HelpText {
    base: Div,
    text: SharedString,
}

impl HelpText {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            base: div(),
            text: text.into(),
        }
    }
}

impl Styled for HelpText {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for HelpText {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        self.base
            .text_sm()
            .text_color(cx.theme().content.secondary)
            .child(self.text)
    }
}

pub fn inline_error(text: impl Into<SharedString>) -> InlineError {
    InlineError::new(text)
}

#[derive(IntoElement)]
pub struct InlineError {
    base: Div,
    text: SharedString,
    icon: bool,
}

impl InlineError {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            base: div(),
            text: text.into(),
            icon: true,
        }
    }

    pub fn icon(mut self, icon: bool) -> Self {
        self.icon = icon;
        self
    }
}

impl Styled for InlineError {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for InlineError {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let bg = cx.theme().status.error.bg;
        let fg = cx.theme().status.error.fg;

        self.base
            .flex()
            .items_center()
            .gap_2()
            .text_sm()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(bg)
            .text_color(fg)
            .when(self.icon, |this| {
                this.child(icon(IconName::Close).size(px(14.)).color(fg))
            })
            .child(self.text)
    }
}

pub fn form_row(label: impl Into<SharedString>, control: impl IntoElement) -> FormRow {
    FormRow::new(label, control)
}

#[derive(IntoElement)]
pub struct FormRow {
    base: Div,

    label: SharedString,
    control: gpui::AnyElement,

    help: Option<gpui::AnyElement>,
    error: Option<gpui::AnyElement>,

    label_width: gpui::Pixels,
    label_color: Option<Hsla>,

    validation: Option<ValidationState>,
    validation_icon_size: Option<gpui::Pixels>,
}

impl FormRow {
    pub fn new(label: impl Into<SharedString>, control: impl IntoElement) -> Self {
        Self {
            base: div(),
            label: label.into(),
            control: control.into_any_element(),
            help: None,
            error: None,
            label_width: px(140.),
            label_color: None,
            validation: None,
            validation_icon_size: None,
        }
    }

    pub fn help(mut self, help: impl IntoElement) -> Self {
        self.help = Some(help.into_any_element());
        self
    }

    pub fn error(mut self, error: impl IntoElement) -> Self {
        self.error = Some(error.into_any_element());
        self
    }

    pub fn label_width(mut self, width: gpui::Pixels) -> Self {
        self.label_width = width;
        self
    }

    pub fn label_color(mut self, color: impl Into<Hsla>) -> Self {
        self.label_color = Some(color.into());
        self
    }

    pub fn validation(mut self, state: ValidationState) -> Self {
        self.validation = Some(state);
        self
    }

    /// Override the size of the [`ValidationStateIcon`] rendered next to the control.
    ///
    /// By default, [`FormRow`] uses the icon's own default size.
    pub fn validation_icon_size(mut self, size: gpui::Pixels) -> Self {
        self.validation_icon_size = Some(size);
        self
    }
}

impl ParentElement for FormRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for FormRow {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for FormRow {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let label_color = self
            .label_color
            .unwrap_or_else(|| cx.theme().content.secondary);

        let help_or_error = if let Some(error) = self.error {
            Some(error)
        } else {
            self.help
        };

        let validation_icon_size = self.validation_icon_size;
        let validation_icon = self.validation.map(|state| {
            let mut icon = validation_state_icon(state);
            if let Some(size) = validation_icon_size {
                icon = icon.size(size);
            }
            icon.into_any_element()
        });

        self.base
            .w_full()
            .flex()
            .items_start()
            .gap_3()
            .child(
                div()
                    .w(self.label_width)
                    .pt(px(8.))
                    .text_sm()
                    .text_color(label_color)
                    .child(label(self.label).inherit_color(true).ellipsis(true)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.control)
                            .children(validation_icon),
                    )
                    .children(help_or_error),
            )
    }
}
