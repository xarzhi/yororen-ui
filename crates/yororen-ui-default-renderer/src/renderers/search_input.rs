//! `SearchInputRenderer` — visual side of `SearchInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` (the inner
//! painter) plus a search icon at the leading edge and a
//! clear-button at the trailing edge. Escape key clears the
//! value.

use std::any::Any;
use std::sync::Arc;

use gpui::{
    div, px, AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window,
};
use gpui::prelude::FluentBuilder;
use yororen_ui_core::headless::icon::{icon, IconSource};
use yororen_ui_core::headless::search_input::SearchInputProps;
use yororen_ui_core::headless::text_input::{Escape, TextInputState};
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::icon::DefaultIcon;
use crate::renderers::spec::Edges;
use crate::renderers::text_input::{
    start_cursor_blink, wire_input_keyboard, TextInputElement, TextInputRenderState,
    TextInputRenderer,
};

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
    fn hover_border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla;
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
    fn hover_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn icon_color(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    fn fg(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    fn min_height(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.search_input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme.get_number("tokens.control.search_input.horizontal_padding").unwrap_or(0.0) as f32),
            px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn input_gap(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.search_input.input_gap").unwrap_or(0.0) as f32)
    }
    fn icon_size(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.search_input.icon_size").unwrap_or(0.0) as f32)
    }
}

pub fn arc_search_input<T: SearchInputRenderer + 'static>(r: T) -> Arc<dyn SearchInputRenderer> {
    Arc::new(r)
}

pub trait DefaultSearchInput: Sized {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement;
}

impl DefaultSearchInput for SearchInputProps {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement {
        let theme_arc = cx.theme().clone();
                let r: Arc<dyn SearchInputRenderer> = cx
            .renderer_arc::<markers::SearchInput, dyn SearchInputRenderer>()
            .expect("SearchInputRenderer registered").clone();
        let theme = &*theme_arc;

        let id = self.id.clone();
        let placeholder_str = self.placeholder.clone();
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let on_clear = self.on_clear.clone();

        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(placeholder_str);
            s.on_change = on_change.clone();
            s.on_submit = on_submit.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
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
        let icon_color = r.icon_color(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let input_gap = r.input_gap(&render_state, theme);
        let icon_size = r.icon_size(&render_state, theme);

        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color: theme.get_color("content.tertiary").unwrap_or_default(),
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
            value_override: None,
        };

        let base: Stateful<Div> = div()
            .id(id.clone())
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
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                gpui::CursorStyle::Arrow
            } else {
                gpui::CursorStyle::IBeam
            });

        let focused_div: Stateful<Div> = base.track_focus(&focus_handle);

        // Search-specific: Escape clears the value.
        let state_for_escape = state.clone();
        let on_change_for_escape = on_change.clone();
        let keyed = wire_input_keyboard(focused_div, state.clone(), focus_handle.clone(), disabled, on_submit)
            .on_action(move |_: &Escape, _window, cx| {
                if disabled {
                    return;
                }
                let before = state_for_escape.read(cx).value.clone();
                state_for_escape.update(cx, |s, cx| {
                    s.value.clear();
                    s.caret = 0;
                    s.selection_start = 0;
                    s.selection_end = 0;
                    cx.notify();
                });
                if let Some(cb) = on_change_for_escape.as_ref() {
                    let after = state_for_escape.read(cx).value.clone();
                    if before != after {
                        cb(&after, _window, cx);
                    }
                }
            });

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);

        // Compose children: search icon, inner element, clear button.
        let state_for_clear = state.clone();
        let on_change_for_clear = on_change.clone();
        let on_clear_clone = on_clear.clone();
        let final_div = keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(
                icon(
                    "search-input-leading-icon",
                    IconSource::Builtin("search".into()),
                    &mut *cx,
                )
                .size(icon_size)
                .color(text_color)
                .default_render(&mut *cx, window),
            )
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .when(!state_for_clear.read(cx).value.is_empty(), |d| {
                d.child(
                    div()
                        .id("search-input-clear")
                        .size(icon_size)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(icon_color)
                        .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                            // The × clears the live `state.value`
                            // first (so the input visually empties
                            // and the icon disappears via the
                            // `.when(...)` re-eval), then fires
                            // `on_change` with "", then
                            // `on_clear` for the caller to do
                            // extra work.
                            state_for_clear.update(cx, |s, cx| {
                                s.value.clear();
                                s.caret = 0;
                                s.selection_start = 0;
                                s.selection_end = 0;
                                cx.notify();
                            });
                            if let Some(cb) = on_change_for_clear.as_ref() {
                                cb("", window, cx);
                            }
                            if let Some(cb) = on_clear_clone.as_ref() {
                                cb(window, cx);
                            }
                        })
                        .child(
                            icon(
                                "search-input-clear-icon",
                                IconSource::Builtin("close".into()),
                                &mut *cx,
                            )
                            .size(icon_size)
                            .color(icon_color)
                            .default_render(&mut *cx, window),
                        ),
                )
            });

        final_div.into_any_element()
    }
}
