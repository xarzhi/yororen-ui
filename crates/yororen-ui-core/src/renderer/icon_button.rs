//! `IconButtonRenderer` — visual contract for `IconButton`.
//!
//! Trait surface is **just** `compose`. The renderer takes the
//! full `IconButtonProps` and returns a fully-built
//! `Stateful<Div>`. Headless layers `on_click` on top.

use std::any::Any;
use std::sync::Arc;

use gpui::{App, Div, FocusHandle, Stateful};

use crate::headless::icon_button::IconButtonProps;
use crate::renderer::variant::{ActionVariantKind, VariantStyle};

/// Projection of `IconButtonProps` used by built-in renderers
/// when they want to factor out helpers. Not part of the
/// `IconButtonRenderer` trait surface.
#[derive(Clone, Debug, Default)]
pub struct IconButtonRenderState {
    pub variant: ActionVariantKind,
    pub disabled: bool,
    pub has_custom_bg: bool,
    pub has_custom_hover_bg: bool,
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

pub trait IconButtonRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for an icon button.
    fn compose(
        &self,
        props: &IconButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div>;
}
