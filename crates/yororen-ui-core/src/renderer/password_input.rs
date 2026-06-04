//! `PasswordInputRenderer` — visual side of `PasswordInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct PasswordInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait PasswordInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
    fn toggle_icon_size(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenPasswordInputRenderer;

impl PasswordInputRenderer for TokenPasswordInputRenderer {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            state.custom_bg.unwrap_or(theme.surface.base)
        }
    }
    fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.border.muted
        } else {
            state.custom_border.unwrap_or(theme.border.default)
        }
    }
    fn focus_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        state.custom_focus_border.unwrap_or(theme.border.focus)
    }
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else {
            state.custom_fg.unwrap_or(theme.content.primary)
        }
    }
    fn min_height(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn padding(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn border_radius(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn toggle_icon_size(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_sm
    }
}

pub fn arc_password_input<T: PasswordInputRenderer + 'static>(
    r: T,
) -> Arc<dyn PasswordInputRenderer> {
    Arc::new(r)
}
