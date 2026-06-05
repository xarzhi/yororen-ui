//! `CheckboxRenderer` — the visual side of `Checkbox`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct CheckboxRenderState {
    pub checked: bool,
    pub disabled: bool,
    pub has_custom_tone: bool,
}

pub trait CheckboxRenderer: Any + Send + Sync {
    fn box_size(&self, state: &CheckboxRenderState, theme: &Theme) -> Pixels;
    fn check_size(&self, state: &CheckboxRenderState, theme: &Theme) -> Pixels;
    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn check_fg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &CheckboxRenderState, theme: &Theme) -> f32;
}

pub struct TokenCheckboxRenderer;

impl CheckboxRenderer for TokenCheckboxRenderer {
    fn box_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.box_size
    }
    fn check_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.check_size
    }
    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else if state.checked {
            theme.action.primary.bg
        } else {
            theme.surface.base
        }
    }
    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.bg
        } else {
            theme.border.default
        }
    }
    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.hover_bg
        } else {
            theme.surface.hover
        }
    }
    fn check_fg(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.fg
    }
    fn focus_color(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &CheckboxRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_checkbox<T: CheckboxRenderer + 'static>(r: T) -> Arc<dyn CheckboxRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultCheckbox` — `headless::CheckboxProps` sugar.
// =====================================================================

use gpui::{prelude::FluentBuilder, div, App, ParentElement, Stateful, Styled, px};
use yororen_ui_core::headless::checkbox::CheckboxProps;

use crate::theme::ActiveTheme;

pub trait DefaultCheckbox: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultCheckbox for CheckboxProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &dyn CheckboxRenderer = &**theme
            .renderers
            .get_checkbox()
            .expect("CheckboxRenderer registered");
        let state = CheckboxRenderState {
            checked: self.checked,
            disabled: self.disabled,
            has_custom_tone: false,
        };
        let bg = r.box_bg(&state, theme);
        let border = r.box_border(&state, theme);
        let size = r.box_size(&state, theme);
        let check_size = r.check_size(&state, theme);
        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border)
            .size(size)
            .rounded(px(4.))
            .flex()
            .items_center()
            .justify_center();
        if self.checked {
            el = el.child(div().bg(border).size(check_size).rounded(px(2.)));
        }
        self.apply(el)
    }
}
