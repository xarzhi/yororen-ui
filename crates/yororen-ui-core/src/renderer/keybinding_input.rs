//! `KeybindingInputRenderer` — visual contract for `KeybindingInput`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{AnyElement, App, Hsla, Window};

use crate::headless::keybinding_input::KeybindingInputProps;

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
    fn compose(
        &self,
        props: &KeybindingInputProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement;
}
