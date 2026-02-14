use gpui::{
    AnyView, AppContext, Hsla, IntoElement, ParentElement, Render, RenderOnce, Styled, div,
};

use crate::theme::ActiveTheme;

/// Defines the placement position of a tooltip relative to its trigger element.
pub enum TooltipPlacement {
    /// Automatically determines the best placement based on available space.
    Auto,
    /// Positions the tooltip above the trigger element.
    Top,
    /// Positions the tooltip to the right of the trigger element.
    Right,
    /// Positions the tooltip below the trigger element.
    Bottom,
    /// Positions the tooltip to the left of the trigger element.
    Left,
}

/// Creates a new tooltip with text content.
///
/// Use `.placement()` to control positioning and `.bg()`/`.text_color()` for customization.
/// The tooltip is typically used with `.with_tooltip()` on interactive elements.
///
/// # Example
/// ```rust
/// use yororen_ui::component::{button, tooltip, TooltipPlacement};
///
/// let btn = button()
///     .child("Hover me")
///     .with_tooltip(tooltip("Helpful information").placement(TooltipPlacement::Bottom));
/// ```
pub fn tooltip(content: impl Into<String>) -> Tooltip {
    Tooltip::text(content)
}

/// A tooltip component that displays contextual information on hover.
///
/// Tooltips are typically used with `.with_tooltip()` on interactive elements like buttons or icons.
/// The tooltip will automatically position itself based on available space, or you can specify
/// a fixed placement using `.placement()`.
#[derive(IntoElement)]
pub struct Tooltip {
    content: String,
    placement: TooltipPlacement,
    bg: Option<Hsla>,
    text_color: Option<Hsla>,
}

struct TooltipView {
    content: String,
    bg: Option<Hsla>,
    text_color: Option<Hsla>,
}

impl Tooltip {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            placement: TooltipPlacement::Auto,
            bg: None,
            text_color: None,
        }
    }

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    pub fn build(self) -> impl Fn(&mut gpui::Window, &mut gpui::App) -> AnyView {
        let content = self.content;
        let _placement = self.placement;
        let bg = self.bg;
        let text_color = self.text_color;
        move |_, cx| {
            cx.new(|_| TooltipView {
                content: content.clone(),
                bg,
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
            .bg(self.bg.unwrap_or_else(|| theme.action.neutral.bg))
            .text_color(self.text_color.unwrap_or_else(|| theme.action.neutral.fg))
            .child(self.content.clone())
    }
}

impl RenderOnce for Tooltip {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div().child(self.content)
    }
}
