//! Headless `tooltip` — hover + delay state.

use std::sync::Arc;

use gpui::{
    App, AppContext, Bounds, Div, ElementId, Entity, InteractiveElement, Pixels, Size, Stateful,
    Window,
};

pub type TooltipCloseCallback = Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TooltipPlacement {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone)]
pub struct TooltipState {
    pub open: bool,
    pub placement: TooltipPlacement,
    pub delay_ms: u64,
    /// Anchor bounds — written by the renderer during prepaint.
    pub anchor_bounds: Option<Bounds<Pixels>>,
    pub content_size: Option<Size<Pixels>>,
    on_close: Option<TooltipCloseCallback>,
}

impl TooltipState {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|_| Self {
            open: false,
            placement: TooltipPlacement::Bottom,
            delay_ms: 400,
            anchor_bounds: None,
            content_size: None,
            on_close: None,
        })
    }

    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn set_placement(&mut self, p: TooltipPlacement) {
        self.placement = p;
    }
    pub fn set_delay_ms(&mut self, d: u64) {
        self.delay_ms = d;
    }
    pub fn set_on_close<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(&mut Window, &mut App),
    {
        self.on_close = Some(Arc::new(f));
    }
    pub fn invoke_close(&self, window: &mut Window, cx: &mut App) {
        if let Some(f) = &self.on_close {
            f(window, cx);
        }
    }
}

#[derive(Clone)]
pub struct TooltipProps {
    pub id: ElementId,
    pub state: Entity<TooltipState>,
    pub text: String,
    /// `true` if the caller supplied a custom background color
    /// (consumed by `TooltipRenderer.has_custom_bg`).
    pub has_custom_bg: bool,
    /// `true` if the caller supplied a custom foreground color
    /// (consumed by `TooltipRenderer.has_custom_fg`).
    pub has_custom_fg: bool,
}

pub fn tooltip(
    id: impl Into<ElementId>,
    text: impl Into<String>,
    state: Entity<TooltipState>,
) -> TooltipProps {
    TooltipProps {
        id: id.into(),
        state,
        text: text.into(),
        has_custom_bg: false,
        has_custom_fg: false,
    }
}

impl TooltipProps {
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn has_custom_fg(mut self, v: bool) -> Self {
        self.has_custom_fg = v;
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the tooltip using the registered `TooltipRenderer`.
    /// The renderer reads `state` to decide visibility; the headless
    /// layer only attaches the element id.
    pub fn render(self, cx: &gpui::App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::tooltip::TooltipRenderer;
        use crate::renderer::markers::Tooltip as TooltipMarker;

        let r: &Arc<dyn TooltipRenderer> = cx
            .renderer_arc::<TooltipMarker, dyn TooltipRenderer>()
            .expect("TooltipRenderer registered");
        let div = r.compose(&self, cx);
        self.apply(div)
    }
}
