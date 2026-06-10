//! Headless `skeleton` — placeholder shape. No state.

use std::sync::Arc;

use gpui::{Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct SkeletonProps {
    pub id: ElementId,
    /// `true` → block (filled rect); `false` → single line.
    pub block: bool,
    /// Only meaningful when `block == true`. `true` → square
    /// corners; `false` → rounded.
    pub block_sharp: bool,
    /// Legacy alias for `block_sharp` when callers think in terms
    /// of "rounded vs square". Equivalent to `!block_sharp`.
    pub rounded: bool,
}

pub fn skeleton(id: impl Into<ElementId>, _cx: &mut gpui::App) -> SkeletonProps {
    SkeletonProps {
        id: id.into(),
        block: false,
        block_sharp: false,
        rounded: false,
    }
}

impl SkeletonProps {
    pub fn block(mut self, v: bool) -> Self {
        self.block = v;
        self
    }
    pub fn block_sharp(mut self, v: bool) -> Self {
        self.block_sharp = v;
        self.rounded = !v;
        self
    }
    pub fn rounded(mut self, v: bool) -> Self {
        self.rounded = v;
        self.block_sharp = !v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the skeleton using the registered `SkeletonRenderer`.
    /// Returns a `Div`; caller chains `.w(...)` / `.h(...)` for
    /// explicit sizing.
    pub fn render(self, cx: &gpui::App) -> Div {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::Skeleton as SkeletonMarker;
        use crate::renderer::skeleton::SkeletonRenderer;

        let r: &Arc<dyn SkeletonRenderer> = cx
            .renderer_arc::<SkeletonMarker, dyn SkeletonRenderer>()
            .expect("SkeletonRenderer registered");
        r.compose(&self, cx)
    }
}
