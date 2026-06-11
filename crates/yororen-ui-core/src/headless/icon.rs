//! Headless `icon` — a `gpui::svg` with id. No state.

use gpui::{AnyElement, App, Div, ElementId, Hsla, InteractiveElement, IntoElement, SharedString, Stateful, Styled};
use crate::renderer::RendererContext;

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

    /// Render the icon through the registered `IconRenderer`.
    ///
    /// This is the preferred entry point; it lets themes control the
    /// default size / colour. The no-argument [`Self::render_legacy`]
    /// is kept for backward compatibility.
    pub fn render(self, cx: &App) -> AnyElement {
        let r = cx
            .renderer_arc::<crate::renderer::markers::Icon, dyn crate::renderer::icon::IconRenderer>()
            .expect("IconRenderer registered");
        r.compose(&self, cx)
    }

    /// Legacy no-context render. Uses hard-coded defaults.
    #[deprecated(note = "use `.render(cx)` so themes can supply defaults")]
    pub fn render_legacy(self) -> gpui::AnyElement {
        let path = match &self.source {
            IconSource::Builtin(name) => gpui::SharedString::from(format!("icons/{}.svg", name)),
            IconSource::Resource(path) => path.clone(),
        };
        let size = self.size.unwrap_or(gpui::px(14.0));
        let color = self.color.unwrap_or_else(|| gpui::rgb(0x0A0A0A).into());
        gpui::svg()
            .path(path)
            .size(size)
            .id(self.id)
            .text_color(color)
            .into_any_element()
    }
}
