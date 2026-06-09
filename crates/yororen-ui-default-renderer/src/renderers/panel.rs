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

pub use yororen_ui_core::renderer::panel::{PanelRenderState, PanelRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

/// State passed to a `PanelRenderer`. The flags indicate whether
/// the caller supplied explicit overrides for each visual property;
/// the renderer can choose to honour or ignore them.
pub struct TokenPanelRenderer;

impl PanelRenderer for TokenPanelRenderer {
    fn bg(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        // The has_custom_bg flag is honoured by the Panel builder
        // which sets the explicit `.bg(...)`; here we always read
        // the default from the theme.
        theme.get_color("surface.raised").unwrap_or_default()
    }
    fn border(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn padding(&self, _state: &PanelRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32,
        ))
    }
    fn border_radius(&self, _state: &PanelRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(0.0) as f32)
    }
    fn shadow_alpha(&self, _state: &PanelRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_2").unwrap_or_default().a
    }
}

pub fn arc_panel<T: PanelRenderer + 'static>(r: T) -> Arc<dyn PanelRenderer> {
    Arc::new(r)
}
