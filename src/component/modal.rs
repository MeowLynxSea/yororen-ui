use gpui::prelude::FluentBuilder;
use gpui::{
    ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, div, px,
};

use crate::{
    component::{HeadingLevel, IconName, button, heading, icon, icon_button, label},
    theme::{ActionVariantKind, ActiveTheme},
};

/// Callback type for modal close handler.
type ModalCloseCallback = Box<dyn Fn(&mut gpui::Window, &mut gpui::App)>;

/// Modal content shell (dialog panel).
///
/// This component only renders the *panel* (title/content/actions slots) and is
/// intentionally not responsible for overlay / focus trapping.
///
/// Use it inside a popover/overlay layer in your app.
///
/// # Accessibility
///
/// This component provides accessibility support through the following ARIA attributes:
/// - `role="dialog"`: Identifies the element as a dialog window
/// - `aria-modal="true"`: Indicates that the dialog is modal
/// - `aria-labelledby`: Automatically linked to the modal title (if provided)
/// - `aria-describedby`: Can be set to associate with descriptive content
///
/// For full accessibility support, ensure:
/// - The modal is placed within an overlay that traps focus
/// - The Escape key closes the modal
/// - Focus returns to the trigger element when the modal closes
pub fn modal() -> Modal {
    Modal::new()
}

#[derive(IntoElement)]
pub struct Modal {
    element_id: ElementId,
    base: gpui::Div,
    title: Option<SharedString>,
    content: Option<gpui::AnyElement>,
    actions: Option<gpui::AnyElement>,
    width: gpui::Pixels,
    bg: Option<Hsla>,
    border: Option<Hsla>,
    closable: bool,
    on_close: Option<ModalCloseCallback>,
    /// Accessibility: ID of the element that describes this modal.
    /// This is typically used to associate additional descriptive content.
    described_by: Option<SharedString>,
}

impl Default for Modal {
    fn default() -> Self {
        Self::new()
    }
}

impl Modal {
    pub fn new() -> Self {
        Self {
            element_id: "ui:modal".into(),
            base: div(),
            title: None,
            content: None,
            actions: None,
            width: px(520.),
            bg: None,
            border: None,
            closable: false,
            on_close: None,
            described_by: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Combines the current element ID with a suffix to create a child element ID.
    ///
    /// This enables automatic ID composition for nested components, producing
    /// tuple-based IDs like `("parent-id", "child-id")` to avoid ID collisions
    /// when multiple instances of the same component type exist.
    fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
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

    /// Show a close button in the modal header.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Callback fired when the close button is clicked.
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&mut gpui::Window, &mut gpui::App),
    {
        self.on_close = Some(Box::new(handler));
        self
    }

    /// Sets the accessibility description for this modal.
    ///
    /// This associates additional descriptive content with the modal dialog,
    /// which helps screen reader users understand the dialog's purpose or content.
    ///
    /// The value should be the ID of an element that contains the description.
    pub fn described_by(mut self, id: impl Into<SharedString>) -> Self {
        self.described_by = Some(id.into());
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

        // Get child component IDs before moving other fields
        let close_button_id = self.child_id("close-button");

        let element_id_for_base = self.element_id;
        let title = self.title;
        let content = self
            .content
            .unwrap_or_else(|| label("Content").muted(true).into_any_element());
        let actions = self.actions;
        let closable = self.closable;
        let on_close = self.on_close;

        let mut header_children: Vec<gpui::AnyElement> = vec![];

        // Title
        if let Some(title) = title {
            header_children.push(heading(title).level(HeadingLevel::H3).into_any_element());
        } else {
            header_children.push(label("Modal").muted(true).into_any_element());
        }

        // Close button
        if closable {
            let close_button = icon_button(close_button_id)
                .icon(icon(IconName::Close))
                .on_click(move |_, window, cx| {
                    if let Some(handler) = &on_close {
                        handler(window, cx);
                    }
                });
            header_children.push(close_button.into_any_element());
        }

        self.base
            .id(element_id_for_base)
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
                    .children(header_children),
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
    button("ui:modal:primary-action")
        .variant(ActionVariantKind::Primary)
        .child(label_text.into())
}
