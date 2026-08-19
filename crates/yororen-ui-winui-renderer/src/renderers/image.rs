//! `WinUIImageRenderer` — default `ImageRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, InteractiveElement, ParentElement, Stateful, Styled};

use yororen_ui_core::headless::image::ImageProps;

pub use yororen_ui_core::renderer::image::{ImageRenderState, ImageRenderer};

pub struct WinUIImageRenderer;

impl ImageRenderer for WinUIImageRenderer {
    fn compose(&self, props: &ImageProps, _cx: &App) -> Stateful<Div> {
        let img = gpui::img(props.source.as_gpui_source());
        gpui::div().id(props.id.clone()).child(img.size_full())
    }
}

pub fn arc_image<T: ImageRenderer + 'static>(r: T) -> Arc<dyn ImageRenderer> {
    Arc::new(r)
}
