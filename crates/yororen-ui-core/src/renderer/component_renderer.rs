//! Unified `ComponentRenderer<S>` trait — the foundation of the v0.4
//! renderer architecture.
//!
//! P0-4 introduces this trait to consolidate the 38 nearly-identical
//! `XxxRenderer` traits that previously existed as separate files.
//! The new design is:
//!
//! 1. Each component still has a strongly-typed marker trait
//!    (`ButtonRenderer`, `IconButtonRenderer`, …) that is a thin
//!    alias for `ComponentRenderer<XxxRenderState>`. This keeps
//!    `cx.theme().renderers.button` type-safe at the call site.
//! 2. `ComponentRenderer<S: RenderState>` is the single trait body
//!    carrying the per-component methods (`bg`, `fg`, `hover_bg`,
//!    `border`, `border_radius`, `padding`, …). All `XxxRenderer`
//!    traits delegate to it via blanket impls.
//! 3. `RendererRegistry` is migrating from "40+ named `Arc<dyn …>`"
//!    fields to a single `HashMap<TypeId, Arc<dyn Any>>` keyed by
//!    the renderer's render-state type. The 40+ `with_<x>` setters
//!    are preserved as ergonomic thin wrappers around the typed
//!    map API. See [`RendererRegistry::with_component`] for the new
//!    generic setter.
//!
//! This file documents the intent; the actual blanket impl lives in
//! each `XxxRenderer` module.

use std::fmt::Debug;
use std::hash::Hash;

use gpui::{Hsla, Pixels};

use super::spec::{BorderSpec, Edges, ShadowSpec};

/// Marker trait for a component's render-state struct.
///
/// A render state is the read-only context a renderer needs: which
/// variant, hovered, focused, disabled, etc. The fields are kept
/// deliberately minimal — a renderer can read more from the
/// surrounding `Theme` if it needs to.
pub trait RenderState: Clone + Debug + Default + Send + Sync + 'static {}

/// The single trait body that powers every `XxxRenderer`.
///
/// The `S` type parameter is the component's render state. Methods
/// return `Hsla` / `Pixels` / `BorderSpec` etc. and are read by the
/// builder at render time.
///
/// # Implementing
///
/// ```ignore
/// use yororen_ui_core::renderer::{ComponentRenderer, RenderState};
/// use yororen_ui_core::theme::Theme;
///
/// #[derive(Clone, Debug, Default)]
/// pub struct MyRenderState { /* ... */ }
/// impl RenderState for MyRenderState {}
///
/// pub struct MyRenderer;
///
/// impl ComponentRenderer<MyRenderState> for MyRenderer {
///     fn bg(&self, _state: &MyRenderState, _theme: &Theme) -> Hsla { gpui::rgb(0x000000).into() }
///     // …
/// }
/// ```
pub trait ComponentRenderer<S: RenderState>: Send + Sync {
    /// Foreground / background pair as a single convenience method.
    /// Default impls return transparent for the unset side so a
    /// renderer can override only the field it cares about.
    fn bg(&self, state: &S, theme: &Theme) -> Hsla {
        let _ = (state, theme);
        Hsla::default()
    }
    fn fg(&self, state: &S, theme: &Theme) -> Hsla {
        let _ = (state, theme);
        Hsla::default()
    }
    fn border(&self, state: &S, theme: &Theme) -> Option<BorderSpec> {
        let _ = (state, theme);
        None
    }
    fn border_radius(&self, state: &S, theme: &Theme) -> Pixels {
        let _ = (state, theme);
        Pixels::default()
    }
    fn padding(&self, state: &S, theme: &Theme) -> Edges<Pixels> {
        let _ = (state, theme);
        Edges::default()
    }
    fn shadow(&self, state: &S, theme: &Theme) -> Option<ShadowSpec> {
        let _ = (state, theme);
        None
    }
    fn hover_bg(&self, state: &S, theme: &Theme) -> Hsla {
        let _ = (state, theme);
        Hsla::default()
    }
    fn disabled_opacity(&self, _state: &S) -> f32 {
        1.0
    }
}

use crate::theme::Theme;
