//! `IconRenderer` — visual contract for `Icon`.
//!
//! Trait surface is just `compose`. The renderer decides how to
//! resolve the icon source, default size, and default color from
//! the theme.

use std::any::Any;

use gpui::{AnyElement, App};

use crate::headless::icon::IconProps;

/// Projection of `IconProps` used by built-in renderers when they
/// want to share helpers. Not part of the `IconRenderer` trait
/// surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct IconRenderState {
    pub has_custom_color: bool,
    pub has_custom_size: bool,
}

pub trait IconRenderer: Any + Send + Sync {
    /// Build the full `AnyElement` for the icon (typically a
    /// `gpui::Svg` with size and color applied).
    fn compose(&self, props: &IconProps, cx: &App) -> AnyElement;
}
