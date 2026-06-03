//! Click-outside detection for floating UI components (popovers,
//! dropdown menus, modals).
//!
//! gpui-ce 0.3.3 already provides `InteractiveElement::on_mouse_down_out`
//! (used by `Popover` in `component/popover.rs`). This module adds
//! three higher-level conveniences on top of it:
//!
//! - [`on_click_outside`]: a single `Box<dyn Fn>` callback with a
//!   `(&MouseDownEvent, &mut Window, &mut App)` signature, matching
//!   the rest of the `EventCallback` family.
//! - [`ClickOutsideGuard`]: a small RAII-style guard that arms the
//!   trap on construction and disarms it on drop, so a caller can
//!   build a "click outside closes me" behaviour without manually
//!   threading a boolean through state.
//!
//! These helpers are pure infrastructure — they don't render any
//! pixels. Drop them into a `RenderOnce` body to attach a
//! click-outside handler to the root element of a floating
//! component.

use std::sync::Arc;

use gpui::{
    App, Div, InteractiveElement, IntoElement, MouseDownEvent, ParentElement, RenderOnce,
    StyleRefinement, Styled, Window, div,
};

/// Click-outside callback. Mirrors the closure shape used by the
/// rest of the a11y helpers.
pub type ClickOutsideCallback = Arc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

/// A small wrapper that installs an `on_mouse_down_out` handler on a
/// `Div`. This is the building block used by `Popover`,
/// `DropdownMenu`, and (in Phase G) `Modal`.
///
/// `Popover` historically did this inline:
///
/// ```ignore
/// .on_mouse_down_out(move |_ev, window, cx| { ... })
/// ```
///
/// The `ClickOutsideGuard` variant is for cases where the handler
/// needs to be conditionally armed/disarmed (e.g. only when the
/// popover is open). Use [`ClickOutsideGuard::arm`] / [`ClickOutsideGuard::disarm`]
/// at the right points in the render path.
pub struct ClickOutsideGuard {
    /// `true` if the guard is currently armed (will fire on next
    /// click outside). Construction sets this to `true`; call
    /// [`disarm`](Self::disarm) to silence.
    pub armed: bool,
    /// Callback fired on a click outside the owning element.
    pub on_outside: Option<ClickOutsideCallback>,
}

impl ClickOutsideGuard {
    /// Build a new armed guard with the given callback.
    pub fn new(on_outside: ClickOutsideCallback) -> Self {
        Self {
            armed: true,
            on_outside: Some(on_outside),
        }
    }

    /// Build a disarmed guard. Useful when the caller wants to
    /// install the handler up-front and arm it later (e.g. on the
    /// first interaction).
    pub fn disarmed() -> Self {
        Self {
            armed: false,
            on_outside: None,
        }
    }

    /// Arm the guard. Subsequent clicks outside the element will
    /// fire the callback.
    pub fn arm(&mut self) {
        self.armed = true;
    }

    /// Disarm the guard. The callback will not fire until
    /// [`arm`](Self::arm) is called again.
    pub fn disarm(&mut self) {
        self.armed = false;
    }

    /// Apply the guard to a `Div`. If the guard is armed, attaches
    /// the `on_mouse_down_out` handler; otherwise returns the
    /// `Div` unchanged.
    pub fn apply(self, target: Div) -> Div {
        if !self.armed {
            return target;
        }
        let cb = self.on_outside.unwrap_or_else(|| Arc::new(|_, _, _| {}));
        target.on_mouse_down_out(move |ev, window, cx| {
            cb(ev, window, cx);
        })
    }
}

impl Default for ClickOutsideGuard {
    fn default() -> Self {
        Self::disarmed()
    }
}

/// Helper: install a click-outside handler on a `Div` directly,
/// without the guard indirection. Equivalent to calling
/// `div.on_mouse_down_out(...)`.
pub fn on_click_outside<F>(target: Div, handler: F) -> Div
where
    F: 'static + Fn(&MouseDownEvent, &mut Window, &mut App),
{
    target.on_mouse_down_out(handler)
}

/// A `RenderOnce` element that renders nothing and only exists to
/// expose an `on_mouse_down_out` handler. Useful for "click anywhere
/// outside the dialog closes the dialog" patterns where the
/// listener is attached to a sibling of the dialog content (rather
/// than the dialog content itself).
#[derive(IntoElement)]
pub struct ClickOutsideCapture {
    base: Div,
    on_outside: Option<ClickOutsideCallback>,
}

impl ClickOutsideCapture {
    /// Create a new capture element.
    pub fn new() -> Self {
        Self {
            base: div().size_full(),
            on_outside: None,
        }
    }

    /// Set the click-outside callback.
    pub fn on_click_outside<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&MouseDownEvent, &mut Window, &mut App),
    {
        self.on_outside = Some(Arc::new(handler));
        self
    }
}

impl Default for ClickOutsideCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for ClickOutsideCapture {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ClickOutsideCapture {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for ClickOutsideCapture {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl RenderOnce for ClickOutsideCapture {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_outside = self.on_outside;
        let mut base = self.base;
        if let Some(cb) = on_outside {
            base = base.on_mouse_down_out(move |ev, window, cx| {
                cb(ev, window, cx);
            });
        }
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_guard_is_disarmed() {
        let g = ClickOutsideGuard::default();
        assert!(!g.armed);
        assert!(g.on_outside.is_none());
    }

    #[test]
    fn new_guard_is_armed_with_callback() {
        let cb: ClickOutsideCallback = Arc::new(|_, _, _| {});
        let g = ClickOutsideGuard::new(cb);
        assert!(g.armed);
        assert!(g.on_outside.is_some());
    }

    #[test]
    fn arm_and_disarm_toggle() {
        let cb: ClickOutsideCallback = Arc::new(|_, _, _| {});
        let mut g = ClickOutsideGuard::new(cb);
        g.disarm();
        assert!(!g.armed);
        g.arm();
        assert!(g.armed);
    }

    #[test]
    fn disarmed_factory_yields_disarmed() {
        let g = ClickOutsideGuard::disarmed();
        assert!(!g.armed);
        assert!(g.on_outside.is_none());
    }
}
