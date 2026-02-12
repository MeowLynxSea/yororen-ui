use gpui::{
    Animation, AnimationExt, Div, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    Styled, div, ease_in_out, px,
};

use gpui::InteractiveElement;
use gpui::prelude::FluentBuilder;

use crate::theme::ActiveTheme;

/// Creates a new skeleton line element.
pub fn skeleton_line() -> SkeletonLine {
    SkeletonLine::new()
}

#[derive(IntoElement)]
pub struct SkeletonLine {
    element_id: Option<ElementId>,
    base: Div,
    width: Option<Pixels>,
    height: Pixels,
    tone: Option<Hsla>,
}

impl Default for SkeletonLine {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonLine {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),
            width: None,
            height: px(12.),
            tone: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn tone(mut self, tone: impl Into<Hsla>) -> Self {
        self.tone = Some(tone.into());
        self
    }
}

impl ParentElement for SkeletonLine {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for SkeletonLine {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for SkeletonLine {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let element_id = self.element_id;
        let id = element_id.clone().unwrap_or_else(|| ElementId::from("ui:skeleton-line"));
        let theme = cx.theme();

        let base = self.base.id(element_id.unwrap_or_else(|| "".into()))
            .h(self.height)
            .rounded_full()
            .bg(self.tone.unwrap_or(theme.surface.hover))
            .when_some(self.width, |this, w| this.w(w))
            .when(self.width.is_none(), |this| this.w_full());

        base.with_animation(
            (id, "pulse"),
            Animation::new(std::time::Duration::from_millis(1100))
                .repeat()
                .with_easing(ease_in_out),
            move |this, delta| {
                // Animate opacity between 0.55..0.95.
                let t = 0.55 + 0.40 * delta;
                this.opacity(t)
            },
        )
    }
}

/// Creates a new skeleton block element.
pub fn skeleton_block() -> SkeletonBlock {
    SkeletonBlock::new()
}

#[derive(IntoElement)]
pub struct SkeletonBlock {
    element_id: Option<ElementId>,
    base: Div,
    width: Option<Pixels>,
    height: Pixels,
    rounded: bool,
    tone: Option<Hsla>,
}

impl Default for SkeletonBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonBlock {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),
            width: None,
            height: px(80.),
            rounded: true,
            tone: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    pub fn tone(mut self, tone: impl Into<Hsla>) -> Self {
        self.tone = Some(tone.into());
        self
    }
}

impl ParentElement for SkeletonBlock {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for SkeletonBlock {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for SkeletonBlock {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let element_id = self.element_id;
        let id = element_id.clone().unwrap_or_else(|| ElementId::from("ui:skeleton-block"));
        let theme = cx.theme();

        let base = self.base.id(element_id.unwrap_or_else(|| "".into()))
            .h(self.height)
            .when(self.rounded, |this| this.rounded_md())
            .when(!self.rounded, |this| this.rounded_none())
            .bg(self.tone.unwrap_or(theme.surface.hover))
            .when_some(self.width, |this, w| this.w(w))
            .when(self.width.is_none(), |this| this.w_full());

        base.with_animation(
            (id, "pulse"),
            Animation::new(std::time::Duration::from_millis(1200))
                .repeat()
                .with_easing(ease_in_out),
            move |this, delta| {
                let t = 0.55 + 0.40 * delta;
                this.opacity(t)
            },
        )
    }
}
