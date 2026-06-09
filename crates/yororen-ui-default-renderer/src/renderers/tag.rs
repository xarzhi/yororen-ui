//! `TagRenderer` — the visual side of `Tag`.

use std::any::Any;
use std::sync::Arc;

use gpui::{FontWeight, Hsla, Pixels};

pub use yororen_ui_core::renderer::tag::{TagRenderState, TagRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenTagRenderer;

impl TagRenderer for TokenTagRenderer {
    fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.get_color("action.primary.bg").unwrap_or_default()
        } else if state.has_custom_tone {
            // caller provides the tone; we use a placeholder.
            // Real impl: TokenTagRenderer reads the user's custom
            // Hsla from the component state, not here. For now we
            // return the neutral background; the component's render
            // branch overrides when has_custom_tone is set.
            theme.get_color("action.neutral.bg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.bg").unwrap_or_default()
        }
    }

    fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else if state.has_custom_tone {
            theme.get_color("content.on_status").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.fg").unwrap_or_default()
        }
    }

    fn min_height(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.tag.min_height")
                .unwrap_or(0.0) as f32,
        )
    }

    fn padding_x(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32)
    }

    fn font_size(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_xs")
                .unwrap_or(0.0) as f32,
        )
    }

    fn font_weight(&self, _state: &TagRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_medium")
                .unwrap_or(500.0) as f32,
        )
    }

    fn border_radius(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
    }

    fn close_size(&self, _state: &TagRenderState, _theme: &Theme) -> Pixels {
        gpui::px(16.)
    }

    fn close_hover_bg(&self, _state: &TagRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
}

pub fn arc_tag<T: TagRenderer + 'static>(r: T) -> Arc<dyn TagRenderer> {
    Arc::new(r)
}
