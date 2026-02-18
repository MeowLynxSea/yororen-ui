//! Tree item component for displaying a single row in a tree view.
//!
//! This component provides the visual representation of a tree node,
//! including indentation, expand/collapse toggle, icons, and selection states.

use gpui::{
    div, AnyElement, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, Pixels, RenderOnce, Styled,
    StatefulInteractiveElement, prelude::FluentBuilder, px,
};

use crate::component::{checkbox, disclosure};
use crate::theme::ActiveTheme;

use super::tree_data::TreeCheckedState;

/// Creates a new tree item element.
pub fn tree_item(id: impl Into<ElementId>) -> TreeItem {
    TreeItem::new().id(id)
}

/// A row in a tree view, representing a single node.
#[derive(IntoElement)]
pub struct TreeItem {
    element_id: ElementId,
    base: Div,
    depth: usize,
    expanded: bool,
    has_children: bool,
    selected: bool,
    disabled: bool,
    checked: TreeCheckedState,
    show_checkbox: bool,
    icon_element: Option<AnyElement>,
    label_element: Option<AnyElement>,
    secondary: Option<AnyElement>,
    trailing: Option<AnyElement>,
    indent: Pixels,
    hover_bg: Option<Hsla>,
    selected_bg: Option<Hsla>,
    on_context_menu: Option<Box<dyn Fn(&MouseDownEvent, &mut gpui::Window, &mut gpui::App)>>,
}

impl Default for TreeItem {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeItem {
    pub fn new() -> Self {
        Self {
            element_id: "ui:tree-item".into(),
            base: div(),
            depth: 0,
            expanded: false,
            has_children: false,
            selected: false,
            disabled: false,
            checked: TreeCheckedState::Unchecked,
            show_checkbox: false,
            icon_element: None,
            label_element: None,
            secondary: None,
            trailing: None,
            indent: px(20.),
            hover_bg: None,
            selected_bg: None,
            on_context_menu: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn has_children(mut self, has_children: bool) -> Self {
        self.has_children = has_children;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn checked(mut self, checked: TreeCheckedState) -> Self {
        self.checked = checked;
        self
    }

    pub fn show_checkbox(mut self, show: bool) -> Self {
        self.show_checkbox = show;
        self
    }

    pub fn icon(mut self, el: impl IntoElement) -> Self {
        self.icon_element = Some(el.into_any_element());
        self
    }

    pub fn label(mut self, el: impl IntoElement) -> Self {
        self.label_element = Some(el.into_any_element());
        self
    }

    pub fn secondary(mut self, el: impl IntoElement) -> Self {
        self.secondary = Some(el.into_any_element());
        self
    }

    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        self.trailing = Some(el.into_any_element());
        self
    }

    pub fn indent(mut self, indent: Pixels) -> Self {
        self.indent = indent;
        self
    }

    pub fn hover_bg(mut self, bg: impl Into<Hsla>) -> Self {
        self.hover_bg = Some(bg.into());
        self
    }

    pub fn selected_bg(mut self, bg: impl Into<Hsla>) -> Self {
        self.selected_bg = Some(bg.into());
        self
    }

    /// Attach a right-click handler for this row.
    pub fn on_context_menu<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&MouseDownEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_context_menu = Some(Box::new(listener));
        self
    }

    /// Generate a child element ID by combining this component's element ID with a suffix.
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for TreeItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for TreeItem {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for TreeItem {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for TreeItem {}

impl RenderOnce for TreeItem {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        // Extract element_id
        let element_id = self.element_id.clone();

        let theme = cx.theme();
        let depth = self.depth;
        let expanded = self.expanded;
        let has_children = self.has_children;
        let selected = self.selected;
        let disabled = self.disabled;
        let checked = self.checked;
        let show_checkbox = self.show_checkbox;
        let icon_element = self.icon_element;
        let label_element = self.label_element;
        let secondary = self.secondary;
        let trailing = self.trailing;
        let indent = self.indent;
        let hover_bg = self.hover_bg.unwrap_or(theme.surface.hover);
        let selected_bg = self.selected_bg.unwrap_or(theme.action.neutral.active_bg);
        let on_context_menu = self.on_context_menu;

        let is_checked = checked == TreeCheckedState::Checked;

        let disclosure_id: ElementId = (element_id.clone(), "ui:tree-item:disclosure").into();
        let checkbox_id: ElementId = (element_id.clone(), "ui:tree-item:checkbox").into();

        self.base
            .id(element_id.to_string())
            .w_full()
            .min_h(px(32.))
            .pl(indent * depth as f32)
            .pr_3()
            .py_1()
            .rounded_md()
            .flex()
            .items_center()
            .gap_2()
            .when(selected, |this| this.bg(selected_bg))
            .when(!selected, |this| this.hover(|s| s.bg(hover_bg)))
            .when(disabled, |this| this.opacity(0.5))
            .when_some(on_context_menu, |this, handler| {
                this.on_mouse_down(MouseButton::Right, move |ev, window, cx| {
                    cx.stop_propagation();
                    handler(ev, window, cx);
                })
            })
            .when(has_children, |this| {
                this.child(disclosure(disclosure_id).expanded(expanded))
            })
            .when(show_checkbox, |this| {
                this.child(checkbox(checkbox_id).checked(is_checked))
            })
            .children(icon_element)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .items_start()
                    .flex_grow()
                    .children(label_element)
                    .children(secondary.map(|el| {
                        div()
                            .text_sm()
                            .text_color(theme.content.secondary)
                            .child(el)
                    })),
            )
            .children(trailing)
    }
}
