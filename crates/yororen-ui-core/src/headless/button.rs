//! Headless `button` — a clickable, focusable element with no
//! bundled visual.
//!
//! ```ignore
//! div()
//!     .bg(red).rounded(8).p_2()
//!     .apply(button("save", cx).on_click(|ev, w, cx| { ... }))
//!     .child("Save")
//! ```

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use crate::renderer::RendererMarker;

/// Marker for the `Button` component. The renderer crate (built-in
/// or third-party) registers its `ButtonRenderer` impl against
/// this marker; render-time code retrieves it via
/// `cx.renderer_arc::<Button, dyn ButtonRenderer>()`.
pub struct Button;
impl RendererMarker for Button {}

/// Click handler shared by every interactive headless primitive.
pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

/// Returns a `ButtonProps` with a fresh `FocusHandle` minted from `cx`.
///
/// The caller is expected to pass the result to `.apply(div)` (or the
/// renderer's `DefaultButton::default_render`).
pub fn button(id: impl Into<ElementId>, cx: &mut App) -> ButtonProps {
    ButtonProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        on_click: None,
        disabled: false,
        clickable: true,
    }
}

#[derive(Clone)]
pub struct ButtonProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub on_click: Option<ClickCallback>,
    pub disabled: bool,
    pub clickable: bool,
}

impl ButtonProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.on_click = Some(Arc::new(listener));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    /// Headless apply. Wires the focus handle, hover state, and
    /// click handler. Does **not** set any visual property — the
    /// caller's `el` decides colors, padding, radius, etc.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let focus_handle = self.focus_handle.clone();
        let on_click = self.on_click.clone();
        let disabled = self.disabled;
        let clickable = self.clickable;
        let s = el.id(self.id.clone()).track_focus(&focus_handle);
        if clickable
            && !disabled
            && let Some(f) = on_click
        {
            s.on_click(move |ev, window, cx| {
                if disabled {
                    return;
                }
                f(ev, window, cx);
            })
        } else {
            s
        }
    }
}
