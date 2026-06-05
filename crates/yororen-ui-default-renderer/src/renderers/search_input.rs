//! `SearchInputRenderer` — visual side of `SearchInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct SearchInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait SearchInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn icon_color(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn input_gap(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
    fn icon_size(&self, state: &SearchInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenSearchInputRenderer;

impl SearchInputRenderer for TokenSearchInputRenderer {
    fn bg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn focus_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn icon_color(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    fn fg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    fn min_height(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.search_input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.control.search_input.horizontal_padding").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn input_gap(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.search_input.input_gap").unwrap_or(0.0) as f32)
    }
    fn icon_size(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.search_input.icon_size").unwrap_or(0.0) as f32)
    }
}

pub fn arc_search_input<T: SearchInputRenderer + 'static>(r: T) -> Arc<dyn SearchInputRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultSearchInput` — `headless::SearchInputProps` sugar.
// =====================================================================

use gpui::{
    div, App, InteractiveElement, KeyDownEvent, MouseButton, ParentElement, Stateful, Styled,
    Window,
};
use yororen_ui_core::headless::search_input::SearchInputProps;
use yororen_ui_core::renderer::RendererContext;
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultSearchInput: Sized {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div>;
}

impl DefaultSearchInput for SearchInputProps {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn SearchInputRenderer> = cx
            .renderer_arc::<yororen_ui_core::renderer::markers::SearchInput, dyn SearchInputRenderer>(
            )
            .expect("SearchInputRenderer registered");

        let state = self.state.clone();
        let focus_handle = self.focus_handle.clone();
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let on_clear = self.on_clear.clone();
        let placeholder = self.placeholder.clone();
        let disabled = self.disabled;
        let focused = focus_handle.is_focused(window);

        let render_state = SearchInputRenderState {
            disabled,
            focused,
            custom_bg: self.custom_bg,
            custom_border: self.custom_border,
            custom_focus_border: self.custom_focus_border,
            custom_fg: self.custom_text_color,
        };
        let bg = r.bg(&render_state, theme);
        let border_color = if focused {
            r.focus_border(&render_state, theme)
        } else {
            r.border(&render_state, theme)
        };
        let text_color = r.fg(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let input_gap = r.input_gap(&render_state, theme);
        let icon_size = r.icon_size(&render_state, theme);

        let value = state.read(cx).value.clone();
        let has_value = !value.is_empty();
        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .gap(input_gap)
            .text_color(text_color);

        el = el.child(div().size(icon_size).flex().items_center().justify_center().child("🔍"));

        if has_value {
            el = el.child(div().flex_1().child(value));
        } else {
            el = el.child(div().flex_1().text_color(r.icon_color(&render_state, theme)).child(placeholder));
        }

        if has_value {
            let on_clear_clone = on_clear.clone();
            el = el.child(
                div()
                    .size(icon_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("×")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        if let Some(cb) = on_clear_clone.as_ref() {
                            cb(window, cx);
                        }
                    }),
            );
        }

        let focus_for_mouse = focus_handle.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_ev, window, _cx| {
            focus_for_mouse.focus(window);
        });

        if !disabled {
            let state_for_keys = state.clone();
            let on_change_for_keys = on_change.clone();
            let on_submit_for_keys = on_submit.clone();
            el = el.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let keystroke = &ev.keystroke;
                if keystroke.key.as_str() == "enter" {
                    if let Some(cb) = on_submit_for_keys.as_ref() {
                        let snapshot = state_for_keys.read(cx).value.clone();
                        cb(&snapshot, window, cx);
                    }
                    return;
                }
                if keystroke.key.as_str() == "escape" {
                    let new_value = state_for_keys.update(cx, |s, _cx| {
                        s.value.clear();
                        s.caret = 0;
                        s.value.clone()
                    });
                    if let Some(cb) = on_change_for_keys.as_ref() {
                        cb(&new_value, window, cx);
                    }
                    return;
                }
                match keystroke.key.as_str() {
                    "backspace" => {
                        let new_value = state_for_keys.update(cx, |s, _cx| {
                            s.backspace();
                            s.value.clone()
                        });
                        if let Some(cb) = on_change_for_keys.as_ref() {
                            cb(&new_value, window, cx);
                        }
                    }
                    "delete" => {
                        let new_value = state_for_keys.update(cx, |s, _cx| {
                            s.delete_forward();
                            s.value.clone()
                        });
                        if let Some(cb) = on_change_for_keys.as_ref() {
                            cb(&new_value, window, cx);
                        }
                    }
                    _ => {
                        let ch_opt: Option<&str> = keystroke
                            .key_char
                            .as_deref()
                            .filter(|s| !s.is_empty())
                            .or_else(|| {
                                if keystroke.key.is_empty() {
                                    None
                                } else {
                                    Some(keystroke.key.as_str())
                                }
                            });
                        let Some(ch) = ch_opt else { return };
                        if ch.chars().count() == 1
                            && !keystroke.modifiers.control
                            && !keystroke.modifiers.alt
                            && !keystroke.modifiers.platform
                        {
                            let to_insert = ch.to_string();
                            let new_value = state_for_keys.update(cx, |s, _cx| {
                                s.insert_text(&to_insert);
                                s.value.clone()
                            });
                            if let Some(cb) = on_change_for_keys.as_ref() {
                                cb(&new_value, window, cx);
                            }
                        }
                    }
                }
            });
        }

        self.apply(el)
    }
}
