//! `TextInputRenderer` — visual contract for `TextInput`.
//!
//! Trait surface is just `compose`. The renderer takes the
//! full `TextInputProps`, the `cx` / `window` it needs to mint
//! the input state and wire the keymap, and returns a
//! fully-built `AnyElement`. Data flow is one-way: headless
//! hands the renderer everything and gets back the
//! ready-to-paint element.

use std::any::Any;

use gpui::{AnyElement, App, Hsla, Window};

use crate::headless::text_input::TextInputProps;

/// Projection of `TextInputProps` used by built-in renderers
/// when they want to factor out helpers. Not part of the
/// `TextInputRenderer` trait surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextInputRenderer: Any + Send + Sync {
    /// Build the full `AnyElement` for the input (wrapper +
    /// inner `TextInputElement` + keymap + focus tracking).
    fn compose(
        &self,
        props: &TextInputProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement;
}
