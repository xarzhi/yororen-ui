//! `RadioRenderer` — visual contract for `Radio`.
//!
//! Trait surface is just `compose`.

use std::any::Any;

use gpui::{App, Div, FocusHandle, Hsla, Stateful};

use crate::headless::radio::RadioProps;

/// Projection of `RadioProps` used by built-in renderers when
/// they want to factor out helpers. Not part of the
/// `RadioRenderer` trait surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct RadioRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
    pub custom_tone: Option<Hsla>,
}

pub trait RadioRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for a radio.
    fn compose(
        &self,
        props: &RadioProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div>;
}
