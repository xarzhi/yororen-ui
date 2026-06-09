//! `ProgressBarRenderer` — the visual side of `ProgressBar`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::progress::{ProgressBarRenderState, ProgressBarRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenProgressBarRenderer;

impl ProgressBarRenderer for TokenProgressBarRenderer {
    fn track(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }

    fn fill(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }

    fn height(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.progress.bar_default_h")
                .unwrap_or(0.0) as f32,
        )
    }

    fn border_color(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }

    fn border_radius(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
    }
}

pub fn arc_progress_bar<T: ProgressBarRenderer + 'static>(r: T) -> Arc<dyn ProgressBarRenderer> {
    Arc::new(r)
}
