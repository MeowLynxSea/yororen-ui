mod name;

use gpui::{
    ElementId, Hsla, InteractiveElement, IntoElement, Pixels, RenderOnce, SharedString, Styled, px,
    svg,
};
pub use name::*;

use crate::theme::ActiveTheme;

pub fn icon(path: impl Into<IconPath>) -> Icon {
    Icon::new(path)
}

pub enum IconPath {
    Embeded(IconName),
    External(SharedString),
}

impl From<IconName> for IconPath {
    fn from(value: IconName) -> Self {
        Self::Embeded(value)
    }
}

impl From<&'static str> for IconPath {
    fn from(value: &'static str) -> Self {
        Self::External(SharedString::from(value))
    }
}

impl From<String> for IconPath {
    fn from(value: String) -> Self {
        Self::External(SharedString::from(value))
    }
}

impl From<SharedString> for IconPath {
    fn from(value: SharedString) -> Self {
        Self::External(value)
    }
}

impl From<IconPath> for SharedString {
    fn from(value: IconPath) -> SharedString {
        match value {
            IconPath::Embeded(n) => n.into(),
            IconPath::External(n) => n,
        }
    }
}

#[derive(IntoElement)]
pub struct Icon {
    element_id: ElementId,
    path: IconPath,
    size: Pixels,
    color: Option<Hsla>,
    inherit_color: bool,
}

impl Icon {
    pub fn new(path: impl Into<IconPath>) -> Self {
        Self {
            element_id: "ui:icon".into(),
            path: path.into(),
            size: px(14.),
            color: None,
            inherit_color: false,
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

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn inherit_color(mut self, inherit: bool) -> Self {
        self.inherit_color = inherit;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let base = svg().path(self.path).size(self.size).id(self.element_id);
        if let Some(color) = self.color {
            base.text_color(color)
        } else if self.inherit_color {
            base
        } else {
            base.text_color(cx.theme().content.primary)
        }
    }
}

impl From<IconName> for Icon {
    fn from(value: IconName) -> Self {
        Icon::new(value)
    }
}
