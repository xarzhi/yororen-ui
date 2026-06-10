//! `NumberInputRenderer` — visual contract for `NumberInput`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{AnyElement, App, Hsla, Window};

use crate::headless::number_input::NumberInputProps;

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
    fn compose(
        &self,
        props: &NumberInputProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement;
}
