//! Brutalist list renderers: `ListItem`, `TreeItem`, `Form`.

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div, px};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS};

// =====================================================================
// ListItem
// =====================================================================

pub use yororen_ui_core::renderer::list_item::{ListItemRenderState, ListItemRenderer};

pub struct BrutalListItemRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalListItemRenderer {
    pub fn bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn hover_bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn selected_bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn padding(&self, _: &ListItemRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.list_item.padding")
            .unwrap_or(10.0) as f32;
        Edges::symmetric(px(h), px(h / 2.0))
    }
    pub fn min_height(&self, _: &ListItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.list_item.min_height")
            .unwrap_or(36.0) as f32)
    }
    pub fn border_radius(&self, _: &ListItemRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl ListItemRenderer for BrutalListItemRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::list_item::ListItemProps,
        cx: &App,
    ) -> Div {
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
        gpui::div()
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

// =====================================================================
// TreeItem
// =====================================================================

pub use yororen_ui_core::renderer::tree_item::{TreeItemRenderState, TreeItemRenderer};

pub struct BrutalTreeItemRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalTreeItemRenderer {
    pub fn bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn hover_bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn selected_bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn fg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn indent(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        let step = theme
            .get_number("tokens.control.tree_item.indent_step")
            .unwrap_or(16.0) as f32;
        px(state.depth as f32 * step)
    }
    pub fn padding(&self, _: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.tree_item.padding")
            .unwrap_or(8.0) as f32;
        Edges::symmetric(px(p), px(p / 2.0))
    }
    pub fn min_height(&self, _: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tree_item.min_height")
            .unwrap_or(32.0) as f32)
    }
    pub fn chevron_size(&self, _: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.list_item.chevron_size")
            .unwrap_or(16.0) as f32)
    }
}

impl TreeItemRenderer for BrutalTreeItemRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::tree_item::TreeItemProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TreeItemRenderState {
            selected: props.selected,
            expanded: props.expanded,
            depth: props.depth.min(u8::MAX as usize) as u8,
            is_leaf: !props.has_children,
        };
        let bg = if state.selected {
            self.selected_bg(&state, theme)
        } else {
            self.bg(&state, theme)
        };
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let indent = self.indent(&state, theme);
        gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .pl(indent + pad.left)
            .pr(pad.right)
            .min_h(h)
            .child(props.label.clone())
    }
}

// =====================================================================
// Form
// =====================================================================

pub use yororen_ui_core::renderer::form::{FormRenderState, FormRenderer};

pub struct BrutalFormRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalFormRenderer {
    pub fn gap(&self, _: &FormRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.form.gap").unwrap_or(12.0) as f32)
    }
    pub fn label_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn error_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("status.error.fg").unwrap_or(BRUTAL_BORDER)
    }
    pub fn helper_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
    }
}

impl FormRenderer for BrutalFormRenderer {
    fn compose(
        &self,
        _props: &yororen_ui_core::headless::form::FormProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = FormRenderState {};
        let g = self.gap(&state, theme);
        gpui::div().flex().flex_col().gap(g)
    }
}
