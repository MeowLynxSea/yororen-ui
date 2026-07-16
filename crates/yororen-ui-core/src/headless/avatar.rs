//! Headless `avatar` — image or initials in a circle.

use std::sync::Arc;

use gpui::{Div, ElementId, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct AvatarProps {
    pub id: ElementId,
    /// Image shown inside the avatar. Resolved like
    /// [`crate::headless::image::ImageSource::Resource`]: an
    /// `http(s)://` URL is fetched remotely, any other string is a
    /// filesystem path. When `None`, initials / name are rendered
    /// instead.
    pub src: Option<SharedString>,
    pub initials: Option<String>,
    pub name: Option<SharedString>,
    pub size: Option<gpui::Pixels>,
    /// Render as a perfect circle (otherwise a rounded square).
    pub circle: bool,
    /// Show a small status dot (online / away / busy).
    pub has_status: bool,
    /// `true` if the caller supplied a custom background color
    /// (consumed by `AvatarRenderer.has_custom_bg`).
    pub has_custom_bg: bool,
}

pub fn avatar(id: impl Into<ElementId>, _cx: &mut gpui::App) -> AvatarProps {
    AvatarProps {
        id: id.into(),
        src: None,
        initials: None,
        name: None,
        size: None,
        circle: true,
        has_status: false,
        has_custom_bg: false,
    }
}

impl AvatarProps {
    pub fn src(mut self, s: impl Into<SharedString>) -> Self {
        self.src = Some(s.into());
        self
    }
    pub fn initials(mut self, i: impl Into<String>) -> Self {
        self.initials = Some(i.into());
        self
    }
    pub fn name(mut self, n: impl Into<SharedString>) -> Self {
        self.name = Some(n.into());
        self
    }
    pub fn size(mut self, s: impl Into<gpui::Pixels>) -> Self {
        self.size = Some(s.into());
        self
    }
    pub fn circle(mut self, v: bool) -> Self {
        self.circle = v;
        self
    }
    pub fn has_status(mut self, v: bool) -> Self {
        self.has_status = v;
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the avatar using the registered `AvatarRenderer`.
    /// Returns a `Stateful<Div>` with the element id and the
    /// renderer-built avatar (image / initials) and status dot.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::avatar::AvatarRenderer;
        use crate::renderer::markers::Avatar as AvatarMarker;

        let r: &Arc<dyn AvatarRenderer> = cx
            .renderer_arc::<AvatarMarker, dyn AvatarRenderer>()
            .expect("AvatarRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
