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
        raw_hover: true,
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
    /// When `true` (the default), `apply` adds a built-in
    /// opacity-based hover/active feedback so a bare caller
    /// `div()` still has visible interactive response. Set
    /// to `false` when the caller wants full control of the
    /// hover/active styling (e.g. when chaining
    /// `default_render` which already wires its own
    /// `bg → hover_bg → active_bg` transitions, or when the
    /// caller's `div()` already configures `.hover(...)` /
    /// `.active(...)`).
    pub raw_hover: bool,
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

    /// Set whether `apply` should add a built-in opacity
    /// hover/active feedback. Defaults to `true` so a bare
    /// `div()` still has visible interactive response; set
    /// to `false` when the caller wants to own the entire
    /// interactive styling (e.g. chaining `default_render`).
    pub fn raw_hover(mut self, raw: bool) -> Self {
        self.raw_hover = raw;
        self
    }

    /// Headless apply. Wires the focus handle, a default
    /// opacity-based hover/active feedback, and the click
    /// handler. The caller's `el` decides colors, padding,
    /// radius, etc.
    ///
    /// ## Built-in interactive feedback
    ///
    /// `apply` always applies a small opacity dip on hover
    /// (0.9) and a deeper one while pressed (0.85) so every
    /// button has *some* visual response, even when the
    /// caller supplies a bare `div()` and never opts into
    /// the default renderer's `bg/hover_bg/active_bg`. The
    /// feedback is intentionally light (a 10–15% opacity
    /// delta) — it does not compete with the caller's
    /// colors. Apps with a strict "no built-in styles" rule
    /// can call `.raw_hover(false)` before `apply` to opt
    /// out; tests that compare element trees byte-for-byte
    /// also use this knob.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let focus_handle = self.focus_handle.clone();
        let on_click = self.on_click.clone();
        let disabled = self.disabled;
        let clickable = self.clickable;
        let raw_hover = self.raw_hover;
        let s = el.id(self.id.clone()).track_focus(&focus_handle);
        // Default hover/active feedback: light opacity dip.
        // Skipped when the button is disabled (the
        // renderer's `disabled_opacity` already dims it) or
        // when the caller opted out via `.raw_hover(false)`.
        let s = if raw_hover && !disabled {
            s.hover(|mut style| {
                style.opacity = Some(0.9);
                style
            })
            .active(|mut style| {
                style.opacity = Some(0.85);
                style
            })
        } else {
            s
        };
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
