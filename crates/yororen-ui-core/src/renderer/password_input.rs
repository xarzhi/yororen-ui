//! `PasswordInputRenderer` — visual contract for `PasswordInput`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{AnyElement, App, Hsla, Window};

use crate::headless::password_input::PasswordInputProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct PasswordInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait PasswordInputRenderer: Any + Send + Sync {
    fn compose(
        &self,
        props: &PasswordInputProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement;
}
