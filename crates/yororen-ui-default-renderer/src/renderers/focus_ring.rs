//! `FocusRingRenderer` — the visual side of `FocusRing`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

pub use yororen_ui_core::renderer::focus_ring::{FocusRingRenderState, FocusRingRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenFocusRingRenderer;

impl FocusRingRenderer for TokenFocusRingRenderer {
    fn color(&self, _state: &FocusRingRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }

    fn width(&self, _state: &FocusRingRenderState, _theme: &Theme) -> Pixels {
        gpui::px(2.)
    }
}

pub fn arc_focus_ring<T: FocusRingRenderer + 'static>(r: T) -> Arc<dyn FocusRingRenderer> {
    Arc::new(r)
}
