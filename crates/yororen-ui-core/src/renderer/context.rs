//! `RendererContext` — sugar for `cx.register_renderer_arc` and
//! `cx.renderer_arc`.
//!
//! The plain registry API works on `&mut RendererRegistry`
//! directly; the `RendererContext` trait puts the global lookup
//! behind `cx.` so callers don't have to thread the registry
//! around. The trait is implemented for [`App`] and any other
//! gpui context that exposes the `set_global` / `global_mut` /
//! `try_global` primitives.
//!
//! ## Install pattern
//!
//! ```ignore
//! // In a renderer crate's `install` function:
//! pub fn install(cx: &mut App) {
//!     cx.register_renderer_arc::<headless::Button, dyn ButtonRenderer>(
//!         Arc::new(TokenButtonRenderer) as Arc<dyn ButtonRenderer>
//!     );
//!     // ... 37 more
//! }
//! ```
//!
//! ## Retrieve pattern
//!
//! ```ignore
//! // In a renderer crate's `DefaultButton::default_render`:
//! fn default_render(self, cx: &App) -> Stateful<Div> {
//!     let r: &Arc<dyn ButtonRenderer> = cx
//!         .renderer_arc::<headless::Button, dyn ButtonRenderer>()
//!         .expect("ButtonRenderer registered");
//!     // ... use r
//! }
//! ```

use std::sync::Arc;

use gpui::App;

use super::registry::{RendererMarker, RendererRegistry};

/// Sugar trait: register a renderer, or look one up, by marker
/// type.
pub trait RendererContext {
    /// Register `r` as the renderer for marker `T`. Lazily
    /// installs an empty [`RendererRegistry`] global on first
    /// call.
    fn register_renderer_arc<T: RendererMarker, R: ?Sized + Send + Sync + 'static>(
        &mut self,
        r: Arc<R>,
    );

    /// Borrow the renderer for marker `T` downcast to `Arc<R>`.
    /// Returns `None` if no renderer is registered for `T` (or
    /// no registry has been installed).
    fn renderer_arc<T: RendererMarker, R: ?Sized + Send + Sync + 'static>(&self)
    -> Option<&Arc<R>>;

    /// True iff a renderer is registered for marker `T`.
    fn has_renderer<T: RendererMarker>(&self) -> bool;
}

impl RendererContext for App {
    fn register_renderer_arc<T: RendererMarker, R: ?Sized + Send + Sync + 'static>(
        &mut self,
        r: Arc<R>,
    ) {
        // Lazy-install the registry. Subsequent calls just
        // borrow the existing global.
        if self.try_global::<RendererRegistry>().is_none() {
            self.set_global(RendererRegistry::new());
        }
        self.global_mut::<RendererRegistry>()
            .register_arc::<T, R>(r);
    }

    fn renderer_arc<T: RendererMarker, R: ?Sized + Send + Sync + 'static>(
        &self,
    ) -> Option<&Arc<R>> {
        self.try_global::<RendererRegistry>()?
            .renderer_arc::<T, R>()
    }

    fn has_renderer<T: RendererMarker>(&self) -> bool {
        self.try_global::<RendererRegistry>()
            .map(|r| r.contains::<T>())
            .unwrap_or(false)
    }
}

/// One-time global setup: installs an empty [`RendererRegistry`]
/// so subsequent `register_renderer_arc` calls don't pay the
/// "create if missing" branch. Optional — the lazy-init in
/// `RendererContext::register_renderer_arc` covers the same
/// ground, but calling `init_renderer_registry(cx)` once at
/// app boot makes the order of operations explicit.
pub fn init_renderer_registry(cx: &mut App) {
    if cx.try_global::<RendererRegistry>().is_none() {
        cx.set_global(RendererRegistry::new());
    }
}
