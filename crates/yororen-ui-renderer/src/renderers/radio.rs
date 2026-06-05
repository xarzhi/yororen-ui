//! `RadioRenderer` — the visual side of `Radio`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct RadioRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
}

pub trait RadioRenderer: Any + Send + Sync {
    fn ring_size(&self, state: &RadioRenderState, theme: &Theme) -> Pixels;
    fn dot_size(&self, state: &RadioRenderState, theme: &Theme) -> Pixels;
    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_hover_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &RadioRenderState, theme: &Theme) -> f32;
}

pub struct TokenRadioRenderer;

impl RadioRenderer for TokenRadioRenderer {
    fn ring_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.ring_size
    }
    fn dot_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.dot_size
    }
    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.bg
        } else {
            theme.border.default
        }
    }
    fn ring_hover_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn dot_fg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.bg
    }
    fn focus_color(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &RadioRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_radio<T: RadioRenderer + 'static>(r: T) -> Arc<dyn RadioRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultRadio` — `headless::RadioProps` sugar.
// =====================================================================

use gpui::{prelude::FluentBuilder, div, App, ParentElement, Stateful, Styled, px};
use yororen_ui_core::headless::radio::RadioProps;

use crate::theme::ActiveTheme;

pub trait DefaultRadio: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultRadio for RadioProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &dyn RadioRenderer = &**theme
            .renderers
            .get_radio()
            .expect("RadioRenderer registered");
        let state = RadioRenderState {
            checked: self.checked,
            disabled: self.disabled,
            has_custom_tone: false,
        };
        let bg = r.ring_bg(&state, theme);
        let border = r.ring_border(&state, theme);
        let ring_size = r.ring_size(&state, theme);
        let dot_size = r.dot_size(&state, theme);
        let dot_fg = r.dot_fg(&state, theme);
        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border)
            .size(ring_size)
            .rounded(theme.tokens.radii.pill)
            .flex()
            .items_center()
            .justify_center();
        if self.checked {
            el = el.child(
                div()
                    .bg(dot_fg)
                    .size(dot_size)
                    .rounded(theme.tokens.radii.pill),
            );
        }
        self.apply(el)
    }
}
