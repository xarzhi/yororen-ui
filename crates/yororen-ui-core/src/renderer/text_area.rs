//! `TextAreaRenderer` — visual contract for `TextArea`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{AnyElement, App, Hsla, Window};

use crate::headless::text_area::TextAreaProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextAreaRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub has_custom_focus_border: bool,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextAreaRenderer: Any + Send + Sync {
    fn compose(
        &self,
        props: &TextAreaProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement;
}
