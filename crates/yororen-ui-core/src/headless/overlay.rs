//! Headless `overlay` — a scrim + focus trap + Esc handler. The
//! headless layer owns the open state machine; visual lives in the
//! renderer.

use std::sync::Arc;
use crate::renderer::RendererContext;

use gpui::{App, Div, ElementId, FocusHandle, InteractiveElement, Stateful, StatefulInteractiveElement};

/// Reason an overlay was closed. Forwarded to the caller's
/// `on_close` callback so the caller can branch on the cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayCloseReason {
    Escape,
    ScrimClick,
    Programmatic,
}

pub type OverlayCloseCallback =
    Arc<dyn Fn(OverlayCloseReason, &mut gpui::Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct OverlayProps {
    pub id: ElementId,
    pub open: bool,
    pub focus_handle: FocusHandle,
    pub dismiss_on_escape: bool,
    pub dismiss_on_scrim: bool,
    pub on_close: Option<OverlayCloseCallback>,
}

pub fn overlay(id: impl Into<ElementId>, cx: &mut App) -> OverlayProps {
    OverlayProps {
        id: id.into(),
        open: false,
        focus_handle: cx.focus_handle(),
        dismiss_on_escape: true,
        dismiss_on_scrim: true,
        on_close: None,
    }
}

impl OverlayProps {
    pub fn open(mut self, v: bool) -> Self {
        self.open = v;
        self
    }
    pub fn dismiss_on_escape(mut self, v: bool) -> Self {
        self.dismiss_on_escape = v;
        self
    }
    pub fn dismiss_on_scrim(mut self, v: bool) -> Self {
        self.dismiss_on_scrim = v;
        self
    }
    pub fn on_close<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(OverlayCloseReason, &mut gpui::Window, &mut App),
    {
        self.on_close = Some(Arc::new(f));
        self
    }
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the overlay through the registered `OverlayRenderer`.
    ///
    /// The headless layer wires focus tracking and dismissal
    /// handlers (Esc / scrim click) on top of the renderer's scrim.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let r = cx
            .renderer_arc::<crate::renderer::markers::Overlay, dyn crate::renderer::overlay::OverlayRenderer>()
            .expect("OverlayRenderer registered");
        let mut el = r.compose(&self, cx);

        if self.open {
            el = el.track_focus(&self.focus_handle);
            // Dismiss on scrim click when enabled.
            if self.dismiss_on_scrim {
                let on_close = self.on_close.clone();
                el = el.on_click(move |_ev, window, cx| {
                    if let Some(f) = &on_close {
                        f(OverlayCloseReason::ScrimClick, window, cx);
                    }
                });
            }
        }

        el
    }
}
