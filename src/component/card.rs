use gpui::{
    DefiniteLength, Div, Edges, EdgesRefinement, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Styled, div,
};

use crate::theme::ActiveTheme;

/// A padded container card.
///
/// For a "glass" look, combine a translucent card background with a blurred/mica window
/// background (e.g. `WindowBackgroundAppearance::Blurred` on macOS).
///
/// Use `.id()` to set a stable element ID for state management.
pub fn card(id: impl Into<ElementId>) -> Card {
    Card::new().id(id)
}

#[derive(IntoElement)]
pub struct Card {
    element_id: ElementId,
    base: Div,
    bg: Option<Hsla>,
    border: Option<Hsla>,
    glass_alpha: Option<f32>,
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    /// Creates a new card with default styles.
    /// Use `.id()` to set a stable element ID for state management.
    pub fn new() -> Self {
        Self {
            element_id: "ui:card".into(),
            base: div().rounded_lg().border_1().shadow_md().p_4(),
            bg: None,
            border: None,
            glass_alpha: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    /// Override the card background fill.
    ///
    /// Prefer passing a color with an alpha (e.g. `theme.surface.raised.alpha(0.7)`) instead of
    /// applying `.opacity(...)` to the container, which would also fade its content.
    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    /// Override the card border color.
    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border = Some(color.into());
        self
    }

    /// Use a translucent background derived from the current theme.
    ///
    /// This sets the default background to `theme.surface.raised.alpha(alpha)` and slightly fades
    /// the default border. You can still override `bg(...)` / `border(...)` afterwards.
    pub fn glass(mut self, alpha: f32) -> Self {
        self.glass_alpha = Some(alpha.clamp(0.0, 1.0));
        self
    }

    /// Set padding for the card.
    ///
    /// Note: `Card` also implements [`gpui::Styled`], so you can use standard padding style methods
    /// like `.p_3()` / `.px_4()` as well.
    pub fn padding(mut self, padding: Edges<DefiniteLength>) -> Self {
        self.base.style().padding = EdgesRefinement {
            top: Some(padding.top),
            right: Some(padding.right),
            bottom: Some(padding.bottom),
            left: Some(padding.left),
        };
        self
    }

    /// Set uniform padding for the card.
    pub fn padding_all(mut self, value: impl Into<DefiniteLength>) -> Self {
        let value = value.into();
        self.base.style().padding = EdgesRefinement {
            top: Some(value.clone()),
            right: Some(value.clone()),
            bottom: Some(value.clone()),
            left: Some(value),
        };
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Card {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let bg = match (self.bg, self.glass_alpha) {
            (Some(bg), _) => bg,
            (None, Some(alpha)) => theme.surface.raised.alpha(alpha),
            (None, None) => theme.surface.raised,
        };

        let border = match (self.border, self.glass_alpha) {
            (Some(border), _) => border,
            (None, Some(_)) => theme.border.default.alpha(0.6),
            (None, None) => theme.border.default,
        };

        self.base.id(self.element_id)
            .bg(bg)
            .border_color(border)
    }
}
