//! `with_theme` — render a sub-tree with a per-element theme override.
//!
//! The framework's default model is a single `GlobalTheme` that all
//! components read via `cx.theme()`. `with_theme` provides a scoped
//! override: while its inner element's `request_layout`, `prepaint`,
//! and `paint` phases run, `cx.theme()` returns the override; once
//! each phase returns, the previous global theme is restored.
//!
//! This lets the same `button()` / `card()` / etc. factories render
//! twice in the same window with two different skins, which is what
//! the `theme_compare` demo relies on.
//!
//! ```ignore
//! with_theme(mini_theme, || {
//!     div().child(button(...).variant(Primary))
//! })
//! ```
//!
//! Limitations:
//! - The override is process-global during each child phase. If the
//!   inner tree contains another `with_theme`, only the innermost
//!   override is observed during the deepest phase. Compose
//!   carefully.
//! - The previous theme is restored on the normal return path. If
//!   the child panics during a phase, the previous theme is **not**
//!   restored — treat this as a non-panic-safe scope, like
//!   `RefCell::borrow`.

use std::sync::Arc;

use gpui::{AnyElement, App, Element, IntoElement, Window, WindowAppearance};

use crate::theme::{GlobalTheme, Theme, ThemeSet};

/// Render `children` with `theme` temporarily installed as the
/// global theme for the lifetime of the resulting element. See the
/// module docs for usage and caveats.
///
/// The closure runs eagerly (when the returned wrapper is added to
/// its parent) and must produce an `AnyElement`. The closure only
/// **constructs** the child tree — child `RenderOnce` impls (e.g.
/// `Button::render`) run later, when the wrapper element's
/// `request_layout` phase fires, and at that point the override is
/// in effect.
pub fn with_theme<F>(theme: Theme, children: F) -> WithTheme<F>
where
    F: FnOnce() -> AnyElement + 'static,
{
    WithTheme {
        theme: Arc::new(theme),
        children,
    }
}

/// Builder half of `with_theme`. Owns the closure that will produce
/// the inner element. Implements `IntoElement` to flip into the
/// runtime `WithThemeElement` that carries the theme override.
pub struct WithTheme<F> {
    theme: Arc<Theme>,
    children: F,
}

impl<F> IntoElement for WithTheme<F>
where
    F: FnOnce() -> AnyElement + 'static,
{
    type Element = WithThemeElement;

    fn into_element(self) -> Self::Element {
        // Eagerly invoke the closure so the inner element is
        // constructed once. Its children's `RenderOnce::render`
        // is still deferred — those run when the parent later
        // calls `request_layout` on us.
        let child = (self.children)();
        WithThemeElement {
            theme: self.theme,
            child,
        }
    }
}

/// Runtime half of `with_theme`. Wraps the inner `AnyElement` and
/// temporarily installs the override theme for the three element
/// phases where descendants might read `cx.theme()`.
pub struct WithThemeElement {
    theme: Arc<Theme>,
    child: AnyElement,
}

impl IntoElement for WithThemeElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for WithThemeElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let prev = cx.global::<GlobalTheme>().current().clone();
        cx.set_global(GlobalTheme::new_with_themes(
            WindowAppearance::Light,
            ThemeSet::new((*self.theme).clone()),
        ));
        // The 2-arg form on `AnyElement` is the underlying
        // delegation. Child `Component::request_layout` will call
        // each `RenderOnce::render`, which reads `cx.theme()` and
        // bakes in the override palette.
        let layout_id = self.child.request_layout(window, cx);
        cx.set_global(GlobalTheme::new_with_themes(
            WindowAppearance::Light,
            ThemeSet::new((*prev).clone()),
        ));
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let prev = cx.global::<GlobalTheme>().current().clone();
        cx.set_global(GlobalTheme::new_with_themes(
            WindowAppearance::Light,
            ThemeSet::new((*self.theme).clone()),
        ));
        self.child.prepaint(window, cx);
        cx.set_global(GlobalTheme::new_with_themes(
            WindowAppearance::Light,
            ThemeSet::new((*prev).clone()),
        ));
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let prev = cx.global::<GlobalTheme>().current().clone();
        cx.set_global(GlobalTheme::new_with_themes(
            WindowAppearance::Light,
            ThemeSet::new((*self.theme).clone()),
        ));
        self.child.paint(window, cx);
        cx.set_global(GlobalTheme::new_with_themes(
            WindowAppearance::Light,
            ThemeSet::new((*prev).clone()),
        ));
    }
}
