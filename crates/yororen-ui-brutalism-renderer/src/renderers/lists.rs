//! Brutalist list renderers: `ListItem`, `TreeItem`, `Form`.

use gpui::{App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString, Stateful, StatefulInteractiveElement, Styled, Window, prelude::FluentBuilder, px};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

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
            .px(pad.left)
            .py(pad.top)
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
            .get_number("tokens.control.tree_item.chevron_size")
            .unwrap_or_else(|| {
                theme
                    .get_number("tokens.control.list_item.chevron_size")
                    .unwrap_or(18.0)
            }) as f32)
    }
    pub fn disabled_fg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border_radius(&self, _: &TreeItemRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn gap(&self, _: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32)
    }
}

impl TreeItemRenderer for BrutalTreeItemRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::tree_item::TreeItemProps,
        cx: &mut App,
        window: &mut Window,
    ) -> Stateful<Div> {
        use yororen_ui_core::headless::tree_item::{DOUBLE_CLICK_THRESHOLD, LastClick};
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
        let hover_bg = self.hover_bg(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let indent = self.indent(&state, theme);
        let chevron_size = self.chevron_size(&state, theme);
        let gap = self.gap(&state, theme);
        let radius = self.border_radius(&state, theme);

        let chevron_slot = if props.has_children {
            let glyph: SharedString = if props.expanded { "▾".into() } else { "▸".into() };
            let chevron_id: ElementId = format!("{}-chevron", props.id).into();
            let toggle_cb = props.on_toggle.clone();
            let disabled = props.disabled;
            let chevron_color = if disabled {
                self.disabled_fg(&state, theme)
            } else {
                fg
            };
            gpui::div()
                .id(chevron_id)
                .w(chevron_size)
                .h(chevron_size)
                .flex()
                .items_center()
                .justify_center()
                .text_color(chevron_color)
                .when(!disabled, |s| s.cursor_pointer())
                .occlude()
                .child(glyph)
                .on_click(move |ev, window, cx| {
                    if disabled {
                        return;
                    }
                    if let Some(cb) = toggle_cb.as_ref() {
                        cb(ev, window, cx);
                    }
                })
                .into_any_element()
        } else {
            gpui::div()
                .w(chevron_size)
                .h(chevron_size)
                .flex()
                .into_any_element()
        };

        let label_color = if props.disabled {
            self.disabled_fg(&state, theme)
        } else {
            fg
        };

        // Double-click detector — same approach as the default
        // renderer: stamp `now` on every click; if the prior
        // stamp is within the threshold, treat as a
        // double-click and fire `on_double_click` (or
        // `on_toggle` as a v0.2-compatible fallback).
        let last_click = window.use_keyed_state(props.id.clone(), cx, |_, _| LastClick::default());
        let on_click_cb = props.on_click.clone();
        let on_toggle_cb = props.on_toggle.clone();
        let on_double_click_cb = props.on_double_click.clone();
        let disabled = props.disabled;

        let mut row = gpui::div()
            .id(props.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
            .w_full()
            .min_h(h)
            .bg(bg)
            .text_color(label_color)
            .pl(indent + pad.left)
            .pr(pad.right)
            .py(pad.top)
            .rounded(radius);

        if !props.selected && !props.disabled {
            row = row.hover(move |s| s.bg(hover_bg));
        }
        if !props.disabled {
            row = row.cursor_pointer();
        }
        if props.disabled {
            row = row.opacity(0.5);
        }

        if !disabled {
            row = row.on_click(move |ev, window, cx| {
                let prior = last_click.read(cx).clone();
                let is_double = prior.within(DOUBLE_CLICK_THRESHOLD);
                last_click.update(cx, |s, _cx| *s = LastClick::stamp_now());

                if is_double {
                    if let Some(cb) = on_double_click_cb.as_ref() {
                        cb(ev, window, cx);
                    } else if let Some(cb) = on_toggle_cb.as_ref() {
                        cb(ev, window, cx);
                    }
                } else if let Some(cb) = on_click_cb.as_ref() {
                    cb(ev, window, cx);
                }
            });
        }

        row.child(chevron_slot).child(props.label.clone())
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
