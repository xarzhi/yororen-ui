//! `SearchInputRenderer` — visual side of `SearchInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SearchInputRenderState {
    pub disabled: bool,
    pub focused: bool,
}

pub trait SearchInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn icon_color(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn input_gap(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn icon_size(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenSearchInputRenderer;

impl SearchInputRenderer for TokenSearchInputRenderer {
    fn bg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn icon_color(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn fg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn min_height(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.search_input.min_height
    }
    fn padding(&self, _state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.search_input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn border_radius(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn input_gap(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.search_input.input_gap
    }
    fn icon_size(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.search_input.icon_size
    }
}

pub fn arc_search_input<T: SearchInputRenderer + 'static>(r: T) -> Arc<dyn SearchInputRenderer> {
    Arc::new(r)
}
