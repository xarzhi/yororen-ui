//! `TokenVirtualListRenderer` — default `VirtualListRenderer` impl.
//!
//! Returns a scrollable container with a border and subtle
//! background. The caller is responsible for appending visible rows
//! as children after `.render(cx)`.

use std::sync::Arc;

use gpui::{App, Div, Hsla, InteractiveElement, Pixels, Stateful, StatefulInteractiveElement, Styled, div};

use yororen_ui_core::headless::virtual_list::VirtualListProps;
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::virtual_list::{VirtualListRenderState, VirtualListRenderer};

pub struct TokenVirtualListRenderer;

impl TokenVirtualListRenderer {
    pub fn bg(&self, _state: &VirtualListRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border_color(&self, _state: &VirtualListRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn border_radius(&self, _state: &VirtualListRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(4.0) as f32)
    }
}

impl VirtualListRenderer for TokenVirtualListRenderer {
    fn compose(&self, props: &VirtualListProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = VirtualListRenderState {
            item_count: props.item_count,
        };
        let bg = self.bg(&state, theme);
        let border = self.border_color(&state, theme);
        let radius = self.border_radius(&state, theme);

        div()
            .id(props.id.clone())
            .bg(bg)
            .border_1()
            .border_color(border)
            .rounded(radius)
            .overflow_y_scroll()
    }
}

pub fn arc_virtual_list<T: VirtualListRenderer + 'static>(r: T) -> Arc<dyn VirtualListRenderer> {
    Arc::new(r)
}
