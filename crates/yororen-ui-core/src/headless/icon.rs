//! Headless `icon` — a `gpui::svg` with id. No state.

use crate::renderer::RendererContext;
use gpui::{AnyElement, App, Div, ElementId, Hsla, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct IconProps {
    pub id: ElementId,
    pub source: IconSource,
    pub size: Option<gpui::Pixels>,
    /// Display color. `gpui::Svg` does not inherit `text_color`
    /// from a parent element (it reads its own local style at
    /// paint time), so the renderer applies this directly to the
    /// SVG's `style.text.color`. `None` means the icon will not
    /// paint — the caller is expected to pick a color.
    pub color: Option<Hsla>,
}

#[derive(Clone, Debug)]
pub enum IconSource {
    /// One of the universal icons embedded in `yororen-ui-core::assets`.
    Builtin(SharedString),
    /// A resource path resolvable by the application's `AssetSource`.
    Resource(SharedString),
}

pub fn icon(id: impl Into<ElementId>, source: IconSource, _cx: &mut gpui::App) -> IconProps {
    IconProps {
        id: id.into(),
        source,
        size: None,
        color: None,
    }
}

impl IconProps {
    pub fn size(mut self, s: impl Into<gpui::Pixels>) -> Self {
        self.size = Some(s.into());
        self
    }
    pub fn color(mut self, c: impl Into<Hsla>) -> Self {
        self.color = Some(c.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the icon through the registered `IconRenderer`. Themes
    /// supply the default size / colour.
    pub fn render(self, cx: &App) -> AnyElement {
        let r = cx
            .renderer_arc::<crate::renderer::markers::Icon, dyn crate::renderer::icon::IconRenderer>()
            .expect("IconRenderer registered");
        r.compose(&self, cx)
    }
}
