//! `EmptyStateRenderer` — visual side of `EmptyState`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::empty_state::{EmptyStateRenderState, EmptyStateRenderer};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub struct TokenEmptyStateRenderer;

impl EmptyStateRenderer for TokenEmptyStateRenderer {
    fn icon_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    fn title_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.secondary").unwrap_or_default()
    }
    fn body_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    fn padding(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.spacing.inset_lg").unwrap_or(0.0) as f32,
        ))
    }
    fn icon_size(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.sizes.icon_xl").unwrap_or(0.0) as f32)
    }
    fn gap(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32)
    }
}

pub fn arc_empty_state<T: EmptyStateRenderer + 'static>(r: T) -> Arc<dyn EmptyStateRenderer> {
    Arc::new(r)
}
