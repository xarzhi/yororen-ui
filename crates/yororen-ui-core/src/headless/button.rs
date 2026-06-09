//! Headless `button` — a clickable, focusable element with no
//! bundled visual.
//!
//! ```ignore
//! div()
//!     .bg(red).rounded(8).p_2()
//!     .apply(button("save", cx).on_click(|ev, w, cx| { ... }))
//!     .child("Save")
//! ```
//!
//! `apply` is purely a11y: it wires the focus handle, then
//! the click handler. It does **not** inject any visual
//! feedback. The caller's `el` decides colors, padding,
//! radius, hover, active — every visual concern stays
//! with the caller (or the renderer, via `default_render`).

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

// The headless `Button` marker is the same type the
// renderer registry keys on (`core::renderer::markers::Button`).
// Re-exporting it from the headless module keeps the
// `use yororen_ui_core::headless::button::Button as ButtonMarker`
// import path working in renderer code that wants the
// renderer-marker.
pub use crate::renderer::markers::Button;

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
        variant: crate::renderer::ActionVariantKind::default(),
    }
}

#[derive(Clone)]
pub struct ButtonProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub on_click: Option<ClickCallback>,
    pub disabled: bool,
    pub clickable: bool,
    /// Action variant — `Neutral` (default) / `Primary` / `Danger`.
    /// The renderer dispatches to `action.<variant>.{bg,fg}`.
    pub variant: crate::renderer::ActionVariantKind,
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

    pub fn variant(mut self, v: crate::renderer::ActionVariantKind) -> Self {
        self.variant = v;
        self
    }

    /// Wire the headless contract onto the caller's `el`.
    ///
    /// `apply` is purely a11y: it sets the element id,
    /// registers the focus handle, and (if `clickable` and not
    /// `disabled`) attaches the click handler. It does **not**
    /// inject any visual feedback — no opacity dip, no hover /
    /// active style. The caller (or the renderer via
    /// `default_render`) owns the visual.
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
