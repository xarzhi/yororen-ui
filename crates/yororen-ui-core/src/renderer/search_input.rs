//! `SearchInputRenderer` — visual contract for `SearchInput`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{AnyElement, App, Hsla, Window};

use crate::headless::search_input::SearchInputProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct SearchInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait SearchInputRenderer: Any + Send + Sync {
    fn compose(
        &self,
        props: &SearchInputProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement;
}
