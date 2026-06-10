//! `TokenEmptyStateRenderer` — default `EmptyStateRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::empty_state::EmptyStateProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::empty_state::{EmptyStateRenderState, EmptyStateRenderer};

pub struct TokenEmptyStateRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenEmptyStateRenderer {
    pub fn icon_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    pub fn title_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.secondary").unwrap_or_default()
    }
    pub fn body_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    pub fn padding(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(gpui::px(
            theme.get_number("tokens.spacing.inset_lg").unwrap_or(0.0) as f32,
        ))
    }
    pub fn icon_size(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.sizes.icon_xl").unwrap_or(0.0) as f32)
    }
    pub fn gap(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32)
    }
}

impl EmptyStateRenderer for TokenEmptyStateRenderer {
    fn compose(&self, props: &EmptyStateProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = EmptyStateRenderState {};
        let ic = self.icon_color(&state, theme);
        let tc = self.title_color(&state, theme);
        let bc = self.body_color(&state, theme);
        let pad = self.padding(&state, theme);
        let is = self.icon_size(&state, theme);
        let g = self.gap(&state, theme);
        let mut el = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p(pad.top)
            .gap(g);
        if let Some(icon) = &props.icon {
            el = el.child(div().text_color(ic).size(is).child(icon.clone()));
        }
        if let Some(title) = &props.title {
            el = el.child(div().text_color(tc).child(title.clone()));
        }
        if let Some(desc) = &props.description {
            el = el.child(div().text_color(bc).child(desc.clone()));
        }
        el
    }
}

pub fn arc_empty_state<T: EmptyStateRenderer + 'static>(r: T) -> Arc<dyn EmptyStateRenderer> {
    Arc::new(r)
}
