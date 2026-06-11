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
    /// A `gpui::SharedString` interpreted as a resource path.
    Resource(SharedString),
    /// A pre-loaded `gpui::Image` handle — caller constructs it.
    Handle(gpui::Image),
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
