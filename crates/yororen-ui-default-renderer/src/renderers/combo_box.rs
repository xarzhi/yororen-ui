//! `TokenComboBoxRenderer` — default `ComboBoxRenderer` impl.

use std::sync::Arc;

use gpui::{
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};

use yororen_ui_core::headless::combo_box::ComboBoxProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::renderers::text_input::TokenTextInputRenderer;

pub use yororen_ui_core::renderer::combo_box::{ComboBoxRenderState, ComboBoxRenderer};

pub struct TokenComboBoxRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenComboBoxRenderer {
    pub fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    pub fn border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    pub fn focus_border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    pub fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.custom_fg.is_some() {
            state.custom_fg.unwrap()
        } else if state.has_value {
            theme.get_color("content.primary").unwrap_or_default()
        } else {
            theme.get_color("content.tertiary").unwrap_or_default()
        }
    }
    pub fn search_bg(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn padding(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    pub fn border_radius(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

impl ComboBoxRenderer for TokenComboBoxRenderer {
    fn compose(
        &self,
        props: &ComboBoxProps,
        cx: &mut App,
        window: &mut Window,
    ) -> AnyElement {
        use yororen_ui_core::headless::text_input::{TextInputProps, TextInputState};
        use yororen_ui_core::renderer::text_input::TextInputRenderer;
        use yororen_ui_core::theme::ActiveTheme;

        let theme = cx.theme().clone();
        let state_read = props.state.read(cx);
        let state = ComboBoxRenderState {
            open: state_read.is_open(),
            disabled: false,
            has_value: state_read.value.is_some(),
            custom_bg: None,
            custom_border: None,
            custom_focus_border: None,
            custom_fg: None,
        };
        let bg = self.bg(&state, &theme);
        let border = self.border(&state, &theme);
        let _fg = self.fg(&state, &theme);
        let pad = self.padding(&state, &theme);
        let h = self.min_height(&state, &theme);
        let r = self.border_radius(&state, &theme);
        let text = state_read.text.clone();
        let value = state_read.value.clone();
        let options = state_read.options.clone();
        let is_open = state_read.is_open();
        let placeholder = state_read.placeholder.clone();

        // Text-input trigger. Sync the input's value to
        // `combo_state.text`; when text is empty but a
        // value is picked, the input falls back to the
        // value's label.
        let ti_id: gpui::ElementId = (props.id.clone(), "combo-trigger-text-input").into();
        let ti_state = window.use_keyed_state(ti_id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        let display_str: String = if !text.is_empty() {
            text.clone()
        } else if let Some(v) = &value {
            options
                .iter()
                .find(|o| &o.value == v)
                .map(|o| o.label.to_string())
                .unwrap_or_else(|| v.to_string())
        } else {
            String::new()
        };
        ti_state.update(cx, |s, _cx| {
            if s.value != display_str {
                s.set_value(display_str.clone());
            }
        });

        let combo_state_for_text = props.state.clone();

        let ti_props = TextInputProps {
            id: ti_id,
            placeholder: placeholder.to_string(),
            disabled: false,
            max_length: None,
            on_change: Some(Arc::new(move |new: &str, _w: &mut Window, cx: &mut App| {
                combo_state_for_text.update(cx, |s, _cx| {
                    s.text = new.to_string();
                    if !s.is_open() {
                        s.open();
                    }
                });
            })),
            on_submit: None,
            // The text input lives INSIDE the trigger
            // div which already supplies its own border. We
            // want the input to look "naked" — no inner
            // border ring around the editable area.
            has_custom_bg: false,
            has_custom_border: true,
            has_custom_focus_border: true,
            custom_bg: None,
            custom_border: Some(gpui::hsla(0.0, 0.0, 0.0, 0.0)),
            custom_focus_border: Some(gpui::hsla(0.0, 0.0, 0.0, 0.0)),
            custom_text_color: None,
        };
        let ti_element = <TokenTextInputRenderer as TextInputRenderer>::compose(
            &TokenTextInputRenderer,
            &ti_props,
            cx,
            window,
        );

        let hint_color = theme.get_color("content.tertiary").unwrap_or_default();
        let mut trigger: Stateful<Div> = div()
            .flex()
            .items_center()
            .bg(bg)
            .border_1()
            .border_color(border)
            .px(pad.left)
            .min_h(h)
            .rounded(r)
            .id("default-combo-trigger")
            .child(div().flex_1().min_w(px(0.)).child(ti_element))
            .child(
                div()
                    .w(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(hint_color)
                    .child(if is_open { "▴" } else { "▾" }),
            );
        let combo_state_for_open = props.state.clone();
        trigger = trigger.on_click(move |_ev, _window, cx| {
            combo_state_for_open.update(cx, |s, _cx| s.toggle());
        });

        // Filtered options: case-insensitive `contains`.
        let needle = text.to_lowercase();
        let filtered: Vec<(usize, &yororen_ui_core::headless::combo_box::ComboBoxOption)> = options
            .iter()
            .enumerate()
            .filter(|(_, opt)| needle.is_empty() || opt.label.to_lowercase().contains(&needle))
            .collect();

        let mut outer = div().relative().child(trigger);

        if is_open && !filtered.is_empty() {
            let h_f32: f32 = h.into();
            let state_for_close = props.state.clone();
            let mut dropdown: Stateful<Div> = div()
                .id("default-combo-dropdown")
                .absolute()
                .top(px(h_f32 + 4.0))
                .left_0()
                .right_0()
                .bg(theme.get_color("surface.raised").unwrap_or_default())
                .border_1()
                .border_color(border)
                .rounded(r)
                .p(px(4.))
                .flex_col()
                .gap(px(2.))
                .shadow(vec![gpui::BoxShadow {
                    color: gpui::hsla(0.0, 0.0, 0.0, 0.12),
                    offset: gpui::point(px(0.), px(4.)),
                    blur_radius: px(12.),
                    spread_radius: px(0.),
                }])
                .occlude()
                .on_mouse_down_out(move |_ev, _window, cx| {
                    state_for_close.update(cx, |s, _cx| s.close());
                });

            for (orig_i, opt) in filtered.iter() {
                let opt_value = opt.value.clone();
                let opt_label = opt.label.to_string();
                let state_for_opt = props.state.clone();
                let is_selected = value.as_ref() == Some(&opt.value);
                let item_bg = if is_selected {
                    theme.get_color("action.primary.bg").unwrap_or_default()
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let hover_bg = theme.get_color("surface.hover").unwrap_or_default();
                let item_fg = if is_selected {
                    theme.get_color("action.primary.fg").unwrap_or_default()
                } else {
                    theme.get_color("content.primary").unwrap_or_default()
                };
                let opt_label_for_click = opt_label.clone();
                let mut item: Stateful<Div> = div()
                    .id(ElementId::Name(
                        format!("default-combo-opt-{}", orig_i).into(),
                    ))
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(4.))
                    .bg(item_bg)
                    .text_color(item_fg)
                    .hover(move |s| s.bg(hover_bg))
                    .child(opt_label);
                item = item.on_click(move |_ev, _window, cx| {
                    let label_for_text = opt_label_for_click.clone();
                    state_for_opt.update(cx, |s, _cx| {
                        s.set_value(opt_value.clone());
                        s.text = label_for_text;
                        s.close();
                    });
                });
                dropdown = dropdown.child(item);
            }

            outer = outer.child(gpui::deferred(dropdown).with_priority(1));
        }

        outer.into_any_element()
    }
}

pub fn arc_combo_box<T: ComboBoxRenderer + 'static>(r: T) -> Arc<dyn ComboBoxRenderer> {
    Arc::new(r)
}
