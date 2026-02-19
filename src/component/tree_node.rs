//! Tree node component for rendering a single node with its children.

use gpui::{
    Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    StatefulInteractiveElement, Styled, div, px,
};

use crate::theme::ActiveTheme;

use super::tree_data::{FlatTreeNode, TreeCheckedState, TreeNodeData};

/// Creates a new tree node element.
pub fn tree_node(id: impl Into<ElementId>) -> TreeNodeComponent {
    TreeNodeComponent::new().id(id)
}

/// A tree node component that renders a node and its children.
#[derive(IntoElement)]
pub struct TreeNodeComponent {
    element_id: ElementId,
    base: Div,
    node: Option<FlatTreeNode>,
    show_checkbox: bool,
    indent: Pixels,
    draggable: bool,
    selected: bool,
    hover_bg: Option<Hsla>,
    selected_bg: Option<Hsla>,
}

impl Default for TreeNodeComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeNodeComponent {
    pub fn new() -> Self {
        Self {
            element_id: "ui:tree-node".into(),
            base: div(),
            node: None,
            show_checkbox: false,
            indent: px(20.),
            draggable: false,
            selected: false,
            hover_bg: None,
            selected_bg: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn node(mut self, node: FlatTreeNode) -> Self {
        self.node = Some(node);
        self
    }

    pub fn show_checkbox(mut self, show: bool) -> Self {
        self.show_checkbox = show;
        self
    }

    pub fn indent(mut self, indent: Pixels) -> Self {
        self.indent = indent;
        self
    }

    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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
}

impl ParentElement for TreeNodeComponent {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for TreeNodeComponent {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for TreeNodeComponent {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for TreeNodeComponent {}

impl RenderOnce for TreeNodeComponent {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let node = match self.node {
            Some(node) => node,
            None => return div().into_any_element(),
        };

        let theme = cx.theme();
        let element_id = self.element_id;
        let depth = node.depth;
        let _expanded = node.expanded;
        let _has_children = node.has_children;
        let selected = self.selected;
        let checked = node.checked;
        let _show_checkbox = self.show_checkbox;
        let indent = self.indent;
        let hover_bg = self.hover_bg.unwrap_or(theme.surface.hover);
        let selected_bg = self.selected_bg.unwrap_or(theme.action.neutral.active_bg);

        let indent_width = indent * depth as f32;
        let _is_checked = checked == TreeCheckedState::Checked;
        let id_str = element_id.to_string();
        let label_text = node.data.label().to_string();

        let bg_color = if selected { selected_bg } else { hover_bg };

        div()
            .id(id_str)
            .w_full()
            .min_h(px(32.))
            .px_3()
            .py_1()
            .rounded_md()
            .flex()
            .items_center()
            .gap_2()
            .bg(bg_color)
            .child(div().w(indent_width).flex().items_center().justify_center())
            .child(label_text)
            .into_any_element()
    }
}
