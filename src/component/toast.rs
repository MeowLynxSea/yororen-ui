use gpui::prelude::FluentBuilder;
use gpui::{
    AlignSelf, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, div, px,
};

use crate::{
    component::{Icon, IconName, label},
    theme::ActiveTheme,
};

/// Creates a new toast component.
/// Use `.message()` to set the toast message.
///
/// # Example
///
/// ```rust
/// toast()
///     .message("Operation completed")
///     .kind(ToastKind::Success)
/// ```
pub fn toast() -> Toast {
    Toast::new()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToastKind {
    Neutral,
    Success,
    Warning,
    Error,
    Info,
}

#[derive(IntoElement)]
pub struct Toast {
    base: gpui::Div,
    message: Option<SharedString>,
    kind: ToastKind,
    icon: bool,
    wrap: bool,
    bg: Option<Hsla>,
    fg: Option<Hsla>,
    width: Option<Pixels>,
    max_width: Option<Pixels>,
}

impl Default for Toast {
    fn default() -> Self {
        Self::new()
    }
}

impl Toast {
    pub fn new() -> Self {
        Self {
            base: div(),
            message: None,
            kind: ToastKind::Neutral,
            icon: true,
            wrap: false,
            bg: None,
            fg: None,
            width: None,
            max_width: None,
        }
    }

    /// Set the toast message.
    pub fn message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn icon(mut self, icon: bool) -> Self {
        self.icon = icon;
        self
    }

    /// When `false` (default), uses a single line with ellipsis truncation.
    /// When `true`, allows wrapping (useful with `max_width`).
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn fg(mut self, color: impl Into<Hsla>) -> Self {
        self.fg = Some(color.into());
        self
    }

    /// Fix the toast width.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Constrain the toast width while allowing the message to wrap.
    pub fn max_width(mut self, width: Pixels) -> Self {
        self.max_width = Some(width);
        self
    }
}

impl ParentElement for Toast {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Toast {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Toast {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let (bg, fg, icon) = match self.kind {
            ToastKind::Neutral => (theme.surface.raised, theme.content.primary, IconName::Info),
            ToastKind::Success => (
                theme.status.success.bg,
                theme.content.on_status,
                IconName::Check,
            ),
            ToastKind::Warning => (
                theme.status.warning.bg,
                theme.content.on_status,
                IconName::Warning,
            ),
            ToastKind::Error => (
                theme.status.error.bg,
                theme.content.on_status,
                IconName::Close,
            ),
            ToastKind::Info => (
                theme.status.info.bg,
                theme.content.on_status,
                IconName::Info,
            ),
        };

        let bg = self.bg.unwrap_or(bg);
        let fg = self.fg.unwrap_or(fg);

        // In column flex containers, children are often stretched to full width.
        // Toast should shrink to its content by default.
        let mut base = self.base;
        if base.style().align_self.is_none() {
            base.style().align_self = Some(AlignSelf::FlexStart);
        }

        let width = self.width;
        let max_width = self.max_width;
        let constrain_width = width.is_some() || max_width.is_some();

        base.id("ui:toast")
            .px_3()
            .py_2()
            .rounded_md()
            .bg(bg)
            .text_color(fg)
            .shadow_md()
            .flex()
            .items_center()
            .gap_2()
            .when_some(width, |this, width| this.w(width))
            .when(width.is_none(), |this| {
                this.when_some(max_width, |this, max_width| this.max_w(max_width))
            })
            .when(self.icon, |this| {
                // Explicit color to avoid relying on inherited SVG behavior.
                this.child(Icon::new(icon).size(px(14.)).color(fg))
            })
            // Ensure the message uses the same color as the container.
            .when_some(self.message, |this, message| {
                this.child(
                    div()
                        .when(constrain_width, |this| this.flex_1().min_w(px(0.)))
                        .child(label(message).inherit_color(true).ellipsis(!self.wrap)),
                )
            })
    }
}
