//! `CardRenderer` — visual side of `Card`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::card::{CardRenderState, CardRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub struct TokenCardRenderer;

impl CardRenderer for TokenCardRenderer {
    fn bg(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn border(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn padding(&self, _state: &CardRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32,
        ))
    }
    fn border_radius(&self, _state: &CardRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.lg").unwrap_or(0.0) as f32)
    }
    fn shadow_alpha(&self, _state: &CardRenderState, theme: &Theme) -> f32 {
        theme.get_color("shadow.elevation_1").unwrap_or_default().a
    }
}

pub fn arc_card<T: CardRenderer + 'static>(r: T) -> Arc<dyn CardRenderer> {
    Arc::new(r)
}
