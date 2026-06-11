//! `TextRenderer` — visual contract for `Text`.
//!
//! Trait surface is just `compose`. The renderer applies the
//! theme's text size / colour defaults.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::text::TextProps;

/// Projection of `TextProps` used by built-in renderers when they
/// want to share helpers. Not part of the `TextRenderer` trait
/// surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextRenderState {
    pub has_custom_size: bool,
    pub has_custom_color: bool,
}

pub trait TextRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the text span.
    fn compose(&self, props: &TextProps, cx: &App) -> Stateful<Div>;
}
