//! `TokenFocusRingRenderer` — default `FocusRingRenderer` impl.

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, InteractiveElement, Pixels, Stateful, Styled, div, hsla, point};

pub use yororen_ui_core::renderer::focus_ring::{FocusRingRenderState, FocusRingRenderer};
use yororen_ui_core::headless::focus_ring::FocusRingProps;
use yororen_ui_core::theme::Theme;

pub struct TokenFocusRingRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenFocusRingRenderer {
    pub fn color(&self, _state: &FocusRingRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }

    pub fn width(&self, _state: &FocusRingRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.focus_ring.width")
                .unwrap_or(2.0) as f32,
        )
    }
}

impl FocusRingRenderer for TokenFocusRingRenderer {
    fn compose(&self, props: &FocusRingProps, cx: &gpui::App) -> Stateful<gpui::Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = FocusRingRenderState {
            has_custom_color: props.has_custom_color,
        };
        let color = self.color(&state, theme);
        let width = self.width(&state, theme);
        // Draw a ring via box-shadow with no offset and a positive
        // spread radius equal to the ring width. This works for any
        // width (the macro-generated `border_1`…`border_8` helpers
        // do not), and it lays the ring *outside* the element bounds
        // so it never displaces the focused content.
        div()
            .id(props.id.clone())
            .track_focus(&props.focus_handle)
            .shadow(vec![BoxShadow {
                color,
                offset: point(gpui::px(0.), gpui::px(0.)),
                blur_radius: gpui::px(0.),
                spread_radius: width,
            }])
    }
}

pub fn arc_focus_ring<T: FocusRingRenderer + 'static>(r: T) -> Arc<dyn FocusRingRenderer> {
    Arc::new(r)
}
