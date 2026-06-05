//! `NumberInputRenderer` — visual side of `NumberInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct NumberInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait NumberInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn stepper_bg(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn stepper_fg(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn stepper_button_size(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn stepper_icon_size(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &NumberInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenNumberInputRenderer;

impl NumberInputRenderer for TokenNumberInputRenderer {
    fn bg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn stepper_bg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.bg
    }
    fn stepper_fg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.action.neutral.fg
    }
    fn min_height(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.number_input.min_height
    }
    fn padding(&self, _state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.number_input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn stepper_button_size(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.number_input.stepper_button_size
    }
    fn stepper_icon_size(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.number_input.stepper_icon_size
    }
    fn border_radius(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
}

pub fn arc_number_input<T: NumberInputRenderer + 'static>(r: T) -> Arc<dyn NumberInputRenderer> {
    Arc::new(r)
}
