//! Core renderer dispatch — the trait-object store that lets
//! `cx.renderer_arc::<T, R>()` return a typed renderer reference
//! at render time.
//!
//! See [`RendererRegistry`] for storage and the
//! `RendererContext` sugar trait for the `cx.register_renderer_arc`
//! / `cx.renderer_arc` API.

mod context;
mod registry;

pub use context::{init_renderer_registry, RendererContext};
pub use registry::{RendererMarker, RendererRegistry};
