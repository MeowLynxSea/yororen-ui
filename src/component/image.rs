use std::{path::PathBuf, sync::Arc};

use gpui::{
    AnyElement, Div, Image, ImageFormat, IntoElement, ObjectFit, ParentElement, RenderOnce, Styled,
    StyledImage, div, img,
};

use crate::theme::ActiveTheme;

pub fn image(source: impl Into<ImageSource>) -> ImageView {
    ImageView::new(source)
}

pub enum ImageSource {
    Embedded(Arc<Image>),
    Path(PathBuf),
}

impl From<Arc<Image>> for ImageSource {
    fn from(value: Arc<Image>) -> Self {
        Self::Embedded(value)
    }
}

impl From<PathBuf> for ImageSource {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

#[derive(Clone, Copy)]
pub enum ImageFit {
    Contain,
    Cover,
}

impl From<ImageFit> for ObjectFit {
    fn from(value: ImageFit) -> Self {
        match value {
            ImageFit::Contain => ObjectFit::Contain,
            ImageFit::Cover => ObjectFit::Cover,
        }
    }
}

#[derive(IntoElement)]
pub struct ImageView {
    base: Div,
    source: ImageSource,
    fit: ImageFit,
}

impl ImageView {
    pub fn new(source: impl Into<ImageSource>) -> Self {
        Self {
            base: div(),
            source: source.into(),
            fit: ImageFit::Contain,
        }
    }

    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }
}

impl ParentElement for ImageView {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ImageView {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ImageView {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let bg = cx.theme().surface.sunken;
        let fg = cx.theme().content.tertiary;
        let placeholder = move || -> AnyElement {
            div()
                .size_full()
                .rounded_md()
                .bg(bg)
                .flex()
                .items_center()
                .justify_center()
                .text_color(fg)
                .child("Image")
                .into_any_element()
        };

        let fallback = move || -> AnyElement {
            div()
                .size_full()
                .rounded_md()
                .bg(bg)
                .flex()
                .items_center()
                .justify_center()
                .text_color(fg)
                .child("Failed")
                .into_any_element()
        };

        let image = match self.source {
            ImageSource::Embedded(image) => img(image),
            ImageSource::Path(path) => img(path),
        }
        .object_fit(self.fit.into())
        .with_loading(placeholder)
        .with_fallback(fallback)
        .size_full();

        self.base.child(image)
    }
}

pub fn image_from_bytes(bytes: Vec<u8>) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Png, bytes))
}
