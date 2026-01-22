use gpui::{
    AnyElement, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, div,
    prelude::FluentBuilder, px,
};

use crate::theme::ActiveTheme;

/// A virtualization-safe row shell.
///
/// Responsibilities (must-haves):
/// 1) Stable key: `VirtualRow::key(key)` sets `.id((key, "virtual-row"))`.
///    This prevents state bleeding when list virtualization reuses layout slots.
/// 2) Row-local element namespace: the row is rendered inside
///    `window.with_element_namespace((key, "virtual-row-ns"), ...)` so that any
///    `Location::caller()`-based component ids do not collide across recycled rows.
/// 3) Spacing/dividers belong to the shell: callers should render only content.
#[derive(IntoElement)]
pub struct VirtualRow {
    key: Option<ElementId>,
    base: gpui::Div,
    show_divider: bool,
    gap_below: gpui::Pixels,
}

impl Default for VirtualRow {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualRow {
    pub fn new() -> Self {
        Self {
            key: None,
            base: div(),
            show_divider: false,
            gap_below: px(0.),
        }
    }

    /// Provide a stable per-item key.
    pub fn key(mut self, key: impl Into<ElementId>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Whether to render a divider line below this row.
    pub fn divider(mut self, show: bool) -> Self {
        self.show_divider = show;
        self
    }

    /// Extra spacing below the row (outside the row content).
    pub fn gap_below(mut self, gap: gpui::Pixels) -> Self {
        self.gap_below = gap;
        self
    }
}

/// Convenience constructor.
pub fn virtual_row(key: impl Into<ElementId>) -> VirtualRow {
    VirtualRow::new().key(key)
}

impl ParentElement for VirtualRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for VirtualRow {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for VirtualRow {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let key = self
            .key
            .expect("VirtualRow requires a stable key: use virtual_row(key) or .key(key)");

        let show_divider = self.show_divider;
        let gap_below = self.gap_below;
        let divider_color = cx.theme().border.divider;

        // This namespace prevents `Location::caller()` ids inside the row content
        // from colliding with other rows when gpui virtualizes and reuses slots.
        window.with_element_namespace((key.clone(), "virtual-row-ns"), |_window| {
            div()
                .id((key.clone(), "virtual-row"))
                .flex()
                .flex_col()
                .w_full()
                .child(self.base)
                .when(show_divider, move |this| {
                    this.child(div().h(px(1.)).w_full().bg(divider_color))
                })
                .when(gap_below > px(0.), move |this| {
                    this.child(div().h(gap_below))
                })
        })
    }
}
