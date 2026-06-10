//! `TokenListItemRenderer` — default `ListItemRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div};

use yororen_ui_core::headless::list_item::ListItemProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::list_item::{ListItemRenderState, ListItemRenderer};

pub struct TokenListItemRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenListItemRenderer {
    pub fn bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn selected_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    pub fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    pub fn padding(&self, _state: &ListItemRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    pub fn min_height(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.list_item.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
}

impl ListItemRenderer for TokenListItemRenderer {
    fn compose(&self, props: &ListItemProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ListItemRenderState {
            selected: props.selected,
            disabled: props.disabled,
            hovered: false,
        };
        let bg = if state.selected {
            self.selected_bg(&state, theme)
        } else {
            self.bg(&state, theme)
        };
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .p(pad.top)
            .min_h(h)
            .rounded(r)
            .child(props.title.to_string())
    }
}

pub fn arc_list_item<T: ListItemRenderer + 'static>(r: T) -> Arc<dyn ListItemRenderer> {
    Arc::new(r)
}
