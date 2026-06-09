//! Core renderer dispatch — the trait-object store that lets
//! `cx.renderer_arc::<T, R>()` return a typed renderer reference
//! at render time.
//!
//! See [`RendererRegistry`] for storage and the
//! `RendererContext` sugar trait for the `cx.register_renderer_arc`
//! / `cx.renderer_arc` API.

mod context;
pub mod markers;
mod registry;
mod variant;

pub use context::{RendererContext, init_renderer_registry};
pub use markers::*;
pub use registry::{RendererMarker, RendererRegistry};
pub use variant::{
    ActionVariantKind, BuiltinVariantKey, ButtonVariant, GlobalVariantRegistry, TokenVariantStyle,
    VariantKey, VariantRegistry, VariantState, VariantStyle, variant_compose,
};
