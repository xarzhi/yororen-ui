//! `TokenListboxRenderer` — default `ListboxRenderer` impl.
//!
//! Paints a vertical stack of option rows: the highlighted row
//! takes `surface.hover`, the selected row takes
//! `action.primary.bg` + `action.primary.fg`, and disabled
//! rows fade to `content.disabled`. Clicking a row fires
//! `state.pick(value, …)` which writes the value and invokes
//! the user-supplied `on_change` callback.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::headless::listbox::ListboxProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;
use gpui::prelude::FluentBuilder;

pub use yororen_ui_core::renderer::listbox::{ListboxRenderState, ListboxRenderer};

pub struct TokenListboxRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenListboxRenderer {
    pub fn bg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn selected_bg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    pub fn fg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn selected_fg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.fg").unwrap_or_default()
    }
    pub fn disabled_fg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.disabled").unwrap_or_default()
    }
    pub fn padding(&self, _state: &ListboxRenderState, theme: &Theme) -> Edges<Pixels> {
        // Reuse the list_item tokens so the listbox looks
        // consistent with the rest of the lists surfaces.
        Edges::symmetric(
            gpui::px(
                theme
                    .get_number("tokens.control.list_item.horizontal_padding")
                    .unwrap_or_else(|| theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0))
                    as f32,
            ),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32),
        )
    }
    pub fn min_height(&self, _state: &ListboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.list_item.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &ListboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
    pub fn gap(&self, _state: &ListboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.gap_1").unwrap_or(2.0) as f32)
    }
}

impl ListboxRenderer for TokenListboxRenderer {
    fn compose(&self, props: &ListboxProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let read = props.state.read(cx);
        let state = ListboxRenderState {
            row_count: read.options.len(),
        };
        let bg = self.bg(&state, theme);
        let hover_bg = self.hover_bg(&state, theme);
        let selected_bg = self.selected_bg(&state, theme);
        let fg = self.fg(&state, theme);
        let selected_fg = self.selected_fg(&state, theme);
        let disabled_fg = self.disabled_fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let gap = self.gap(&state, theme);

        let highlighted = read.highlighted_index;
        let selected_value = read.selected_value.clone();
        let options = read.options.clone();
        // `Entity` is `Clone`; cloning releases the read borrow
        // so the closure body below can call `state.update(…)`
        // for the click handlers.
        let state_for_click = props.state.clone();
        let _ = read;

        let mut body: Div = div().flex().flex_col().gap(gap).bg(bg).rounded(r).p(px(2.0));

        for (i, opt) in options.iter().enumerate() {
            let is_highlighted = highlighted == Some(i);
            let is_selected = selected_value.as_ref() == Some(&opt.value);
            let row_fg = if opt.disabled {
                disabled_fg
            } else if is_selected {
                selected_fg
            } else {
                fg
            };
            let row_bg = if is_selected {
                selected_bg
            } else if is_highlighted {
                hover_bg
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 0.0)
            };
            let value_for_click = opt.value.clone();
            let mut row: Stateful<Div> = div()
                .id(ElementId::Name(format!("listbox-row-{}", i).into()))
                .flex()
                .items_center()
                .bg(row_bg)
                .text_color(row_fg)
                .px(pad.left)
                .py(pad.top)
                .min_h(h)
                .rounded(r)
                .when(!opt.disabled, |d| d.cursor(CursorStyle::PointingHand))
                .when(opt.disabled, |d| d.opacity(0.6))
                .child(opt.label.to_string());
            if !opt.disabled {
                let state_for_this_row = state_for_click.clone();
                row = row.on_click(move |_ev, window, cx| {
                    state_for_this_row.update(cx, |s, cx_inner| {
                        s.pick(value_for_click.clone(), window, &mut *cx_inner);
                    });
                });
            }
            body = body.child(row);
        }

        div().id(props.id.clone()).child(body)
    }
}

pub fn arc_listbox<T: ListboxRenderer + 'static>(r: T) -> Arc<dyn ListboxRenderer> {
    Arc::new(r)
}