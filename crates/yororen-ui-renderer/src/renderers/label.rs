//! `LabelRenderer` — the visual side of `Label`.

use std::any::Any;
use std::sync::Arc;

use gpui::{FontWeight, Hsla, SharedString};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct LabelRenderState {
    pub muted: bool,
    pub strong: bool,
    pub mono: bool,
    pub inherit_color: bool,
}

pub trait LabelRenderer: Any + Send + Sync {
    fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla;
    fn strong_weight(&self, state: &LabelRenderState, theme: &Theme) -> FontWeight;
    fn family_mono(&self, state: &LabelRenderState, theme: &Theme) -> SharedString;
}

pub struct TokenLabelRenderer;

impl LabelRenderer for TokenLabelRenderer {
    fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla {
        if state.inherit_color {
            // Inherit means "use whatever parent div set". RenderOnce
            // has already painted the base text color; we return a sentinel
            // that the caller compares against to skip .text_color().
            // The cleanest way is to use the theme's content.primary as
            // a no-op color the caller detects by checking the bool.
            // For simplicity here, return the base content color.
            theme.content.primary
        } else if state.muted {
            theme.content.secondary
        } else {
            theme.content.primary
        }
    }

    fn strong_weight(&self, _state: &LabelRenderState, theme: &Theme) -> FontWeight {
        theme.tokens.typography.weight_semibold
    }

    fn family_mono(&self, _state: &LabelRenderState, theme: &Theme) -> SharedString {
        theme.tokens.typography.family_mono.clone()
    }
}

pub fn arc_label<T: LabelRenderer + 'static>(r: T) -> Arc<dyn LabelRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultLabel` — `headless::LabelProps` sugar.
// =====================================================================

use gpui::{prelude::FluentBuilder, div, App, ParentElement, Stateful, Styled};
use yororen_ui_core::headless::label::LabelProps;

use crate::theme::ActiveTheme;

pub trait DefaultLabel: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultLabel for LabelProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &dyn LabelRenderer = &**theme
            .renderers
            .get_label()
            .expect("LabelRenderer registered");
        let state = LabelRenderState {
            muted: self.muted,
            strong: self.strong,
            mono: self.mono,
            inherit_color: self.inherit_color,
        };
        let color = r.color(&state, theme);
        let weight = r.strong_weight(&state, theme);
        let family = r.family_mono(&state, theme);
        let mut el = div();
        if !self.inherit_color {
            el = el.text_color(color);
        }
        if self.strong {
            el = el.font_weight(weight);
        }
        if self.mono {
            el = el.font_family(family);
        }
        el = el.child(self.text.clone());
        self.apply(el)
    }
}
