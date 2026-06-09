//! `DisclosureRenderer` — visual side of `Disclosure`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::disclosure::{DisclosureRenderState, DisclosureRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenDisclosureRenderer;

impl DisclosureRenderer for TokenDisclosureRenderer {
    fn trigger_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    fn trigger_fg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    fn trigger_hover_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    fn min_height(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    fn border_radius(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn chevron_rotation(&self, state: &DisclosureRenderState, _theme: &Theme) -> f32 {
        if state.open { 90.0 } else { 0.0 }
    }
    fn body_padding(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_disclosure<T: DisclosureRenderer + 'static>(r: T) -> Arc<dyn DisclosureRenderer> {
    Arc::new(r)
}
