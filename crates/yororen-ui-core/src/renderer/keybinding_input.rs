//! `KeybindingInputRenderer` — visual side of `KeybindingInput`.

use std::any::Any;

use gpui::{Hsla, Pixels};

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
    fn hover_border(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn kbd_bg(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn kbd_fg(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
}
