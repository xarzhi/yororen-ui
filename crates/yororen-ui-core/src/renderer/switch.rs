//! `SwitchRenderer` — visual contract for `Switch`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{App, Div, FocusHandle, Hsla, Stateful};

use crate::headless::switch::SwitchProps;

/// Projection of `SwitchProps` used by built-in renderers when
/// they want to factor out helpers. Not part of the
/// `SwitchRenderer` trait surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct SwitchRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
    pub custom_tone: Option<Hsla>,
}

pub trait SwitchRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for a switch.
    fn compose(
        &self,
        props: &SwitchProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div>;
}
