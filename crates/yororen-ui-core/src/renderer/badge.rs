//! `BadgeRenderer` — the visual side of `Badge`.

use std::sync::Arc;

use gpui::{FontWeight, Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct BadgeRenderState {
    /// `true` if the user supplied `.tone(...)`.
    pub has_custom_tone: bool,
}

pub trait BadgeRenderer: Send + Sync {
    fn bg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla;
    fn padding_x(&self, state: &BadgeRenderState, theme: &Theme) -> Pixels;
    fn height(&self, state: &BadgeRenderState, theme: &Theme) -> Pixels;
    fn font_size(&self, state: &BadgeRenderState, theme: &Theme) -> Pixels;
    fn font_weight(&self, state: &BadgeRenderState, theme: &Theme) -> FontWeight;
    fn border_radius(&self, state: &BadgeRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenBadgeRenderer;

impl BadgeRenderer for TokenBadgeRenderer {
    fn bg(&self, _state: &BadgeRenderState, theme: &Theme) -> Hsla {
        theme.status.info.bg
    }

    fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            theme.content.on_status
        } else {
            theme.status.info.fg
        }
    }

    fn padding_x(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_sm
    }

    fn height(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.badge.min_height
    }

    fn font_size(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        theme.tokens.typography.font_size_xs
    }

    fn font_weight(&self, _state: &BadgeRenderState, theme: &Theme) -> FontWeight {
        theme.tokens.typography.weight_medium
    }

    fn border_radius(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.pill
    }
}

pub fn arc_badge<T: BadgeRenderer + 'static>(r: T) -> Arc<dyn BadgeRenderer> {
    Arc::new(r)
}
