//! `ToggleButtonRenderer` — visual contract for `ToggleButton`.
//!
//! Trait surface is **just** `compose`. The renderer takes the
//! full `ToggleButtonProps` and returns a fully-built
//! `Stateful<Div>` (visuals + children + hover / active + id +
//! focus). Headless layers `on_click` (firing `on_toggle`) on
//! top.

use std::any::Any;
use std::sync::Arc;

use gpui::{App, Div, FocusHandle, Stateful};

use crate::headless::toggle_button::ToggleButtonProps;
use crate::renderer::variant::{ActionVariantKind, VariantStyle};

/// Projection of `ToggleButtonProps` used by built-in
/// renderers when they want to factor out helpers. Not part of
/// the `ToggleButtonRenderer` trait surface.
#[derive(Clone, Debug, Default)]
pub struct ToggleButtonRenderState {
    pub variant: ActionVariantKind,
    pub selected: bool,
    pub disabled: bool,
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

pub trait ToggleButtonRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for a toggle button.
    fn compose(
        &self,
        props: &ToggleButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div>;
}
