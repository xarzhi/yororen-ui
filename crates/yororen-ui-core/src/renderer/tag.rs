//! `TagRenderer` — the visual side of `Tag`.

use std::any::Any;
use std::sync::Arc;

use gpui::{FontWeight, Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct TagRenderState {
    pub selected: bool,
    pub has_custom_tone: bool,
    pub closable: bool,
}

pub trait TagRenderer: Any + Send + Sync {
    fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn padding_x(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn font_size(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn font_weight(&self, state: &TagRenderState, theme: &Theme) -> FontWeight;
    fn border_radius(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn close_size(&self, state: &TagRenderState, theme: &Theme) -> Pixels;
    fn close_hover_bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla;
}

pub struct TokenTagRenderer;

impl TagRenderer for TokenTagRenderer {
    fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.action.primary.bg
        } else if state.has_custom_tone {
            // caller provides the tone; we use a placeholder.
            // Real impl: TokenTagRenderer reads the user's custom
            // Hsla from the component state, not here. For now we
            // return the neutral background; the component's render
            // branch overrides when has_custom_tone is set.
            theme.action.neutral.bg
        } else {
            theme.action.neutral.bg
        }
    }

    fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.action.primary.fg
        } else if state.has_custom_tone {
            theme.content.on_status
        } else {
            theme.action.neutral.fg
        }
    }

    fn min_height(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.tag.min_height
    }

    fn padding_x(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_sm
    }

    fn font_size(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        theme.tokens.typography.font_size_xs
    }

    fn font_weight(&self, _state: &TagRenderState, theme: &Theme) -> FontWeight {
        theme.tokens.typography.weight_medium
    }

    fn border_radius(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.pill
    }

    fn close_size(&self, _state: &TagRenderState, _theme: &Theme) -> Pixels {
        gpui::px(16.)
    }

    fn close_hover_bg(&self, _state: &TagRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.hover_bg
    }
}

pub fn arc_tag<T: TagRenderer + 'static>(r: T) -> Arc<dyn TagRenderer> {
    Arc::new(r)
}
