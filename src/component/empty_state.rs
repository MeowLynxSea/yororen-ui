use std::panic::Location;

use gpui::prelude::FluentBuilder;
use gpui::{
    Div, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Styled,
    div, px,
};

use crate::component::{Heading, HeadingLevel, Icon, IconName, Label, button, heading, label};
use crate::theme::{ActionVariantKind, ActiveTheme};

#[track_caller]
pub fn empty_state() -> EmptyState {
    EmptyState::new().id(ElementId::from(Location::caller()))
}

#[derive(IntoElement)]
pub struct EmptyState {
    element_id: Option<ElementId>,
    base: Div,
    icon: Option<Icon>,
    title: Option<Heading>,
    description: Option<Label>,
    action: Option<gpui::AnyElement>,
    max_width: Option<Pixels>,
}

impl Default for EmptyState {
    fn default() -> Self {
        Self::new()
    }
}

impl EmptyState {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            element_id: Some(ElementId::from(Location::caller())),
            base: div(),
            icon: Some(crate::component::icon(IconName::Info).size(px(20.))),
            title: None,
            description: None,
            action: None,
            max_width: Some(px(420.)),
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn title(mut self, title: impl Into<gpui::SharedString>) -> Self {
        self.title = Some(heading(title).level(HeadingLevel::H3));
        self
    }

    pub fn description(mut self, description: impl Into<gpui::SharedString>) -> Self {
        self.description = Some(label(description).muted(true));
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    pub fn max_width(mut self, max_width: Pixels) -> Self {
        self.max_width = Some(max_width);
        self
    }
}

impl ParentElement for EmptyState {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for EmptyState {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from("ui:empty-state"));
        let theme = cx.theme();

        let icon = self
            .icon
            .unwrap_or_else(|| crate::component::icon(IconName::Info));

        self.base
            .id(id)
            .flex()
            .flex_col()
            .items_center()
            .text_center()
            .gap_3()
            .px_4()
            .py_6()
            .rounded_md()
            .bg(theme.surface.raised)
            .border_1()
            .border_color(theme.border.default)
            .when_some(self.max_width, |this, w| this.max_w(w))
            .child(
                div()
                    .w(px(44.))
                    .h(px(44.))
                    .rounded_full()
                    .bg(theme.surface.base)
                    .border_1()
                    .border_color(theme.border.muted)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon.color(theme.content.secondary)),
            )
            .children(self.title.map(|t| t.into_any_element()))
            .children(self.description.map(|d| d.into_any_element()))
            .children(
                self.action
                    .map(|a| div().pt_2().child(a).into_any_element()),
            )
    }
}

pub fn empty_state_primary_action(label_text: impl Into<gpui::SharedString>) -> gpui::AnyElement {
    button()
        .variant(ActionVariantKind::Primary)
        .child(label_text.into())
        .into_any_element()
}
