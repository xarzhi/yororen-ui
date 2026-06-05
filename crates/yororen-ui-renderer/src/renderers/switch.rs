//! `SwitchRenderer` — the visual side of `Switch`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SwitchRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
}

pub trait SwitchRenderer: Any + Send + Sync {
    fn track_w(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn track_h(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn knob_size(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &SwitchRenderState, theme: &Theme) -> Pixels;
    fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &SwitchRenderState, theme: &Theme) -> f32;
}

pub struct TokenSwitchRenderer;

impl SwitchRenderer for TokenSwitchRenderer {
    fn track_w(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.track_w
    }
    fn track_h(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.track_h
    }
    fn knob_size(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.knob_size
    }
    fn padding(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.padding
    }

    fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        let accent = theme.action.primary.bg;
        if state.disabled {
            theme.surface.sunken
        } else if state.checked {
            accent
        } else {
            theme.surface.hover
        }
    }
    fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.border.muted
        } else {
            theme.border.default
        }
    }
    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.hover_bg
        } else {
            theme.surface.base
        }
    }
    fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else if state.checked {
            theme.action.primary.fg
        } else {
            theme.content.primary
        }
    }
    fn focus_color(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &SwitchRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_switch<T: SwitchRenderer + 'static>(r: T) -> Arc<dyn SwitchRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultSwitch` — `headless::SwitchProps` sugar.
// =====================================================================

use gpui::{prelude::FluentBuilder, div, App, ParentElement, Stateful, Styled, px};
use yororen_ui_core::headless::switch::SwitchProps;

use crate::theme::ActiveTheme;

pub trait DefaultSwitch: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultSwitch for SwitchProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &dyn SwitchRenderer = &**theme
            .renderers
            .get_switch()
            .expect("SwitchRenderer registered");
        let state = SwitchRenderState {
            checked: self.checked,
            disabled: self.disabled,
            has_custom_tone: false,
        };
        let track = r.track_bg(&state, theme);
        let knob = r.knob_bg(&state, theme);
        let w = r.track_w(&state, theme);
        let h = r.track_h(&state, theme);
        let knob_size = r.knob_size(&state, theme);
        let pad = r.padding(&state, theme);
        let mut el = div()
            .bg(track)
            .w(w)
            .h(h)
            .rounded(theme.tokens.radii.pill)
            .p(pad)
            .flex()
            .items_center();
        if self.checked {
            el = el.justify_end();
        } else {
            el = el.justify_start();
        }
        el = el.child(
            div()
                .bg(knob)
                .size(knob_size)
                .rounded(theme.tokens.radii.pill),
        );
        self.apply(el)
    }
}
