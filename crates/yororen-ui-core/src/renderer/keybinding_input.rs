//! `KeybindingInputRenderer` — visual side of `KeybindingInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct KeybindingInputRenderState {
    pub capturing: bool,
    pub disabled: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait KeybindingInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn kbd_bg(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn kbd_fg(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn kbd_padding(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn kbd_min_width(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
    fn min_height(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
    fn icon_size(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenKeybindingInputRenderer;

impl KeybindingInputRenderer for TokenKeybindingInputRenderer {
    fn bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn kbd_bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn kbd_fg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn kbd_padding(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.keybinding_input.kbd_padding_x,
            theme.tokens.control.keybinding_input.kbd_padding_y,
        )
    }
    fn kbd_min_width(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.keybinding_input.kbd_min_width
    }
    fn min_height(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn border_radius(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.radii.md
    }
    fn icon_size(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.keybinding_input.icon_size
    }
}

pub fn arc_keybinding_input<T: KeybindingInputRenderer + 'static>(
    r: T,
) -> Arc<dyn KeybindingInputRenderer> {
    Arc::new(r)
}
