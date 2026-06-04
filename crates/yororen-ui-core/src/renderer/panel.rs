//! `PanelRenderer` — the visual side of `Panel`.
//!
//! The `Panel` component (defined here) supplies bg, border, radius,
//! in `yororen_ui_core::component::panel`) is the visual "card"
//! primitive that [`Modal`](crate::component::modal::Modal) and
//! other dialog components compose. It carries a renderer trait
//! that themes override via the `RendererRegistry`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

/// State passed to a `PanelRenderer`. The flags indicate whether
/// the caller supplied explicit overrides for each visual property;
/// the renderer can choose to honour or ignore them.
#[derive(Clone, Copy, Debug, Default)]
pub struct PanelRenderState {
    /// `true` if the user supplied `.bg(...)` on the builder.
    pub has_custom_bg: bool,
    /// `true` if the user supplied `.border(...)` on the builder.
    pub has_custom_border: bool,
    /// `true` if the user supplied `.padding(...)` on the builder.
    pub has_custom_padding: bool,
}

pub trait PanelRenderer: Any + Send + Sync {
    fn bg(&self, state: &PanelRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &PanelRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &PanelRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &PanelRenderState, theme: &Theme) -> Pixels;
    fn shadow_alpha(&self, state: &PanelRenderState, theme: &Theme) -> f32;
}

pub struct TokenPanelRenderer;

impl PanelRenderer for TokenPanelRenderer {
    fn bg(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        // The has_custom_bg flag is honoured by the Panel builder
        // which sets the explicit `.bg(...)`; here we always read
        // the default from the theme.
        theme.surface.raised
    }
    fn border(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn padding(&self, _state: &PanelRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(theme.tokens.spacing.inset_md)
    }
    fn border_radius(&self, _state: &PanelRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.lg
    }
    fn shadow_alpha(&self, _state: &PanelRenderState, theme: &Theme) -> f32 {
        theme.shadow.elevation_2.a
    }
}

pub fn arc_panel<T: PanelRenderer + 'static>(r: T) -> Arc<dyn PanelRenderer> {
    Arc::new(r)
}
