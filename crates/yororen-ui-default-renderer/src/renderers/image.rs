//! `TokenImageRenderer` — default `ImageRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, InteractiveElement, ParentElement, Stateful, Styled};

use yororen_ui_core::headless::image::{ImageProps, ImageSource};

pub use yororen_ui_core::renderer::image::{ImageRenderState, ImageRenderer};

pub struct TokenImageRenderer;

impl ImageRenderer for TokenImageRenderer {
    fn compose(&self, props: &ImageProps, _cx: &App) -> Stateful<Div> {
        let img = match &props.source {
            ImageSource::Resource(path) => gpui::img(path.to_string()),
            ImageSource::Handle(handle) => {
                gpui::img(gpui::ImageSource::Image(Arc::new(handle.clone())))
            }
        };
        gpui::div().id(props.id.clone()).child(img.size_full())
    }
}

pub fn arc_image<T: ImageRenderer + 'static>(r: T) -> Arc<dyn ImageRenderer> {
    Arc::new(r)
}
