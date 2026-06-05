//! `FocusRingRenderer` — the visual side of `FocusRing`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct FocusRingRenderState {
    /// `true` if the user supplied `.color(...)`.
    pub has_custom_color: bool,
}

pub trait FocusRingRenderer: Any + Send + Sync {
    fn color(&self, state: &FocusRingRenderState, theme: &Theme) -> Hsla;
    fn width(&self, state: &FocusRingRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenFocusRingRenderer;

impl FocusRingRenderer for TokenFocusRingRenderer {
    fn color(&self, _state: &FocusRingRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }

    fn width(&self, _state: &FocusRingRenderState, _theme: &Theme) -> Pixels {
        gpui::px(2.)
    }
}

pub fn arc_focus_ring<T: FocusRingRenderer + 'static>(r: T) -> Arc<dyn FocusRingRenderer> {
    Arc::new(r)
}
