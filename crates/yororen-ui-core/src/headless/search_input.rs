//! Headless `search_input` — text input with a search-icon
//! and a clear-button. Reuses `TextInputState` (the renderer
//! mints the state via `use_keyed_state`).

use std::sync::Arc;

use gpui::{App, Hsla, Window};

pub type SearchChangeCallback = Arc<dyn Fn(&str, &mut Window, &mut App) + Send + Sync>;
pub type SearchClearCallback = Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct SearchInputProps {
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub value: String,
    pub on_change: Option<SearchChangeCallback>,
    pub on_submit: Option<SearchChangeCallback>,
    pub on_clear: Option<SearchClearCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn search_input(id: impl Into<gpui::ElementId>) -> SearchInputProps {
    SearchInputProps {
        id: id.into(),
        placeholder: "Search…".to_string(),
        disabled: false,
        value: String::new(),
        on_change: None,
        on_submit: None,
        on_clear: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl SearchInputProps {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut gpui::Window, &mut App),
    {
        self.on_submit = Some(Arc::new(f));
        self
    }
    pub fn on_clear<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut gpui::Window, &mut App),
    {
        self.on_clear = Some(Arc::new(f));
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn has_custom_border(mut self, v: bool) -> Self {
        self.has_custom_border = v;
        self
    }
    pub fn has_custom_focus_border(mut self, v: bool) -> Self {
        self.has_custom_focus_border = v;
        self
    }
    pub fn custom_bg(mut self, c: Hsla) -> Self {
        self.custom_bg = Some(c);
        self.has_custom_bg = true;
        self
    }
    pub fn custom_border(mut self, c: Hsla) -> Self {
        self.custom_border = Some(c);
        self.has_custom_border = true;
        self
    }
    pub fn custom_focus_border(mut self, c: Hsla) -> Self {
        self.custom_focus_border = Some(c);
        self.has_custom_focus_border = true;
        self
    }
    pub fn custom_text_color(mut self, c: Hsla) -> Self {
        self.custom_text_color = Some(c);
        self
    }

    /// Render the search input using the registered `SearchInputRenderer`.
    pub fn render(self, cx: &mut gpui::App, window: &mut gpui::Window) -> gpui::AnyElement {
        use crate::headless::icon::{IconSource, icon};
        use crate::headless::text_input::{Escape, TextInputState};
        use crate::headless::text_input_element::{
            TextInputElement, start_cursor_blink, wire_input_keyboard,
        };
        use crate::renderer::RendererContext;
        use crate::renderer::markers::SearchInput as SearchInputMarker;
        use crate::renderer::search_input::{SearchInputRenderState, SearchInputRenderer};
        use crate::renderer::spec::Edges;
        use crate::theme::ActiveTheme;
        use gpui::prelude::FluentBuilder;
        use gpui::{
            CursorStyle, InteractiveElement, IntoElement, MouseButton, ParentElement, Stateful,
            StatefulInteractiveElement, Styled, div, px,
        };
        use std::sync::Arc;

        let theme_arc = cx.theme().clone();
        let r: Arc<dyn SearchInputRenderer> = cx
            .renderer_arc::<SearchInputMarker, dyn SearchInputRenderer>()
            .expect("SearchInputRenderer registered")
            .clone();
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
        let padding: Edges<gpui::Pixels> = r.padding(&render_state, theme);
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

        let base: Stateful<gpui::Div> = div()
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
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            });

        let focused_div: Stateful<gpui::Div> = base.track_focus(&focus_handle);

        let state_for_escape = state.clone();
        let on_change_for_escape = on_change.clone();
        let keyed = wire_input_keyboard(
            focused_div,
            state.clone(),
            focus_handle.clone(),
            disabled,
            on_submit,
        )
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

        let state_for_clear = state.clone();
        let on_change_for_clear = on_change.clone();
        let on_clear_clone = on_clear.clone();

        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(
                icon(
                    "search-input-leading-icon",
                    IconSource::Builtin("search".into()),
                    cx,
                )
                .size(icon_size)
                .color(text_color)
                .render(cx),
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
                                cx,
                            )
                            .size(icon_size)
                            .color(icon_color)
                            .render(cx),
                        ),
                )
            })
            .into_any_element()
    }
}
