//! Headless `icon` — a `gpui::svg` with id. No state.

use gpui::{Div, ElementId, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct IconProps {
    pub id: ElementId,
    pub source: IconSource,
    pub size: Option<gpui::Pixels>,
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
    }
}

impl IconProps {
    pub fn size(mut self, s: impl Into<gpui::Pixels>) -> Self {
        self.size = Some(s.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
