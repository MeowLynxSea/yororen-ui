//! Headless `image` — a `gpui::img` with id + alt. No state.

use std::sync::Arc;

use gpui::{Div, ElementId, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct ImageProps {
    pub id: ElementId,
    pub source: ImageSource,
    pub alt: Option<SharedString>,
}

#[derive(Clone, Debug)]
pub enum ImageSource {
    /// A `gpui::SharedString` interpreted as a resource locator:
    /// `http://` / `https://` strings are fetched over HTTP, any
    /// other string is treated as a filesystem path (relative paths
    /// resolve against the process working directory).
    Resource(SharedString),
    /// A `gpui::SharedString` interpreted as a key into the app's
    /// registered `gpui::AssetSource` (embedded assets).
    Embedded(SharedString),
    /// A pre-loaded `gpui::Image` handle — caller constructs it.
    Handle(gpui::Image),
}

impl ImageSource {
    /// Convert into the [`gpui::ImageSource`] handed to `gpui::img`.
    pub fn as_gpui_source(&self) -> gpui::ImageSource {
        match self {
            ImageSource::Resource(path) => resolve_resource(path.as_ref()),
            ImageSource::Embedded(key) => {
                gpui::ImageSource::Resource(gpui::Resource::Embedded(key.clone()))
            }
            ImageSource::Handle(handle) => gpui::ImageSource::Image(Arc::new(handle.clone())),
        }
    }
}

/// Resolve a resource string (as carried by
/// [`ImageSource::Resource`]) into a [`gpui::ImageSource`].
///
/// `http://` / `https://` strings are fetched over HTTP by gpui;
/// every other string is treated as a filesystem path — relative
/// paths resolve against the process working directory.
///
/// gpui's own `From<&str>` conversion must not be used for plain
/// paths: it routes non-URI strings to the app's `AssetSource`
/// (embedded assets), where ordinary files are never found.
pub fn resolve_resource(resource: &str) -> gpui::ImageSource {
    if resource.starts_with("http://") || resource.starts_with("https://") {
        gpui::ImageSource::from(resource)
    } else {
        gpui::ImageSource::from(std::path::PathBuf::from(resource))
    }
}

pub fn image(id: impl Into<ElementId>, source: ImageSource, _cx: &mut gpui::App) -> ImageProps {
    ImageProps {
        id: id.into(),
        source,
        alt: None,
    }
}

impl ImageProps {
    pub fn alt(mut self, a: impl Into<SharedString>) -> Self {
        self.alt = Some(a.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the image using the registered `ImageRenderer`.
    /// Returns a `Stateful<Div>` wrapping the underlying image
    /// element built by the renderer. The caller may chain `.w(...)` /
    /// `.h(...)` / `.bg(...)` (as a placeholder background while
    /// loading) on the returned div.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::image::ImageRenderer;
        use crate::renderer::markers::Image as ImageMarker;

        let r: &Arc<dyn ImageRenderer> = cx
            .renderer_arc::<ImageMarker, dyn ImageRenderer>()
            .expect("ImageRenderer registered");
        r.compose(&self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_urls_resolve_to_remote_uri() {
        for url in ["http://example.com/a.png", "https://example.com/b.jpg"] {
            match resolve_resource(url) {
                gpui::ImageSource::Resource(gpui::Resource::Uri(uri)) => {
                    assert_eq!(uri.as_ref(), url);
                }
                _ => panic!("expected Uri for {url}"),
            }
        }
    }

    #[test]
    fn plain_strings_resolve_to_filesystem_paths() {
        for path in [
            "./src/assets/images/test.jpg",
            "images/sample.png",
            "/Users/alice/pic.png",
        ] {
            match resolve_resource(path) {
                gpui::ImageSource::Resource(gpui::Resource::Path(p)) => {
                    assert_eq!(p.as_ref(), std::path::Path::new(path));
                }
                _ => panic!("expected Path for {path}"),
            }
        }
    }

    #[test]
    fn embedded_variant_maps_to_asset_source_key() {
        let src = ImageSource::Embedded("icons/logo.svg".into());
        match src.as_gpui_source() {
            gpui::ImageSource::Resource(gpui::Resource::Embedded(key)) => {
                assert_eq!(key.as_ref(), "icons/logo.svg");
            }
            _ => panic!("expected Embedded"),
        }
    }
}
