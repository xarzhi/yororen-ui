//! `RendererRegistry` — the collection of component renderers wired into
//! a `Theme`. Phase B ships a single entry (`button`); Phase C will add
//! the remaining 30+.

use std::sync::Arc;

use super::button::{ButtonRenderer, TokenButtonRenderer};

#[derive(Clone)]
pub struct RendererRegistry {
    pub button: Arc<dyn ButtonRenderer>,
}

impl std::fmt::Debug for RendererRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererRegistry")
            .field("button", &"<dyn ButtonRenderer>")
            .finish()
    }
}

impl Default for RendererRegistry {
    fn default() -> Self {
        Self::token_based()
    }
}

impl RendererRegistry {
    /// All renderers set to the default `TokenXxxRenderer` implementations.
    /// This is the v0.3 / v0.4 visual baseline.
    pub fn token_based() -> Self {
        Self {
            button: Arc::new(TokenButtonRenderer),
        }
    }

    /// Replace the button renderer.
    pub fn with_button(mut self, r: Arc<dyn ButtonRenderer>) -> Self {
        self.button = r;
        self
    }
}
