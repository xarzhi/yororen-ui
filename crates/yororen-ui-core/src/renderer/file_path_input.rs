//! `FilePathInputRenderer` — visual contract for `FilePathInput`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{AnyElement, App, Hsla, Window};

use crate::headless::file_path_input::FilePathInputProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct FilePathInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait FilePathInputRenderer: Any + Send + Sync {
    fn compose(
        &self,
        props: &FilePathInputProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement;
}
