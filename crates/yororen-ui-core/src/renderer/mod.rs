//! Component renderer traits. Phase B spike scope is just `ButtonRenderer`;
//! Phase C generalizes the same shape to 30+ components.
//!
//! See [`button::ButtonRenderer`] for the trait, [`button::TokenButtonRenderer`]
//! for the default implementation that reads from `Theme.tokens`, and
//! [`registry::RendererRegistry`] for the type that lives on `Theme`.

pub mod button;
pub mod registry;
pub mod spec;

pub use button::{ButtonRenderState, ButtonRenderer, TokenButtonRenderer};
pub use registry::RendererRegistry;
pub use spec::{BorderSpec, Edges, IconPosition, ShadowSpec};
