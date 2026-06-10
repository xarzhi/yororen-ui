//! Headless `text_area` — multi-line text input.
//!
//! Reuses `TextInputState` (renderer-minted); the only
//! behavioural difference is that `Enter` inserts a newline
//! instead of firing `on_submit`.

use std::sync::Arc;

use gpui::{App, Hsla};

/// `TextArea` is a multi-line text input. It reuses
/// `TextInputState` for value + caret.
#[derive(Clone)]
pub struct TextAreaProps {
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<super::text_input::TextChangeCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn text_area(id: impl Into<gpui::ElementId>) -> TextAreaProps {
    TextAreaProps {
        id: id.into(),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl TextAreaProps {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn max_length(mut self, v: usize) -> Self {
        self.max_length = Some(v);
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
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

    /// Render the text area using the registered `TextAreaRenderer`.
    pub fn render(self, cx: &mut gpui::App, window: &mut gpui::Window) -> gpui::AnyElement {
        use crate::action_handler;
        use crate::headless::text_area_element::TextAreaElement;
        use crate::headless::text_input::{
            Backspace, Copy, Cut, Delete, End, Enter, Home, Left, Paste, Right, SelectAll,
            SelectLeft, SelectRight, ShowCharacterPalette, TextInputState,
        };
        use crate::headless::text_input_element::start_cursor_blink;
        use crate::renderer::RendererContext;
        use crate::renderer::markers::TextArea as TextAreaMarker;
        use crate::renderer::spec::Edges;
        use crate::renderer::text_area::{TextAreaRenderState, TextAreaRenderer};
        use crate::theme::ActiveTheme;
        use gpui::{
            CursorStyle, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
            MouseMoveEvent, ParentElement, Stateful, StatefulInteractiveElement, Styled, div, px,
        };
        use std::sync::Arc;

        let theme_arc = cx.theme().clone();
        let r: Arc<dyn TextAreaRenderer> = cx
            .renderer_arc::<TextAreaMarker, dyn TextAreaRenderer>()
            .expect("TextAreaRenderer registered")
            .clone();
        let theme = &*theme_arc;

        let id = self.id.clone();
        let placeholder_str = self.placeholder.clone();
        let disabled = self.disabled;
        let max_length = self.max_length;
        let on_change = self.on_change.clone();

        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(placeholder_str);
            s.max_length = max_length;
            s.on_change = on_change.clone();
            s.paste_newlines = true;
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);

        let render_state = TextAreaRenderState {
            disabled,
            focused,
            has_custom_bg: self.has_custom_bg,
            custom_bg: self.custom_bg,
            custom_border: self.custom_border,
            has_custom_focus_border: self.has_custom_focus_border,
            custom_focus_border: self.custom_focus_border,
            custom_text_color: self.custom_text_color,
        };
        let bg = r.bg(&render_state, theme);
        let border_color = if focused {
            r.focus_border(&render_state, theme)
        } else {
            r.border(&render_state, theme)
        };
        let text_color = r.text_color(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding: Edges<gpui::Pixels> = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);

        let hint_color = theme.get_color("content.tertiary").unwrap_or_default();
        let cursor_color = if self.has_custom_focus_border {
            self.custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        };
        let focus_color = theme.get_color("border.focus").unwrap_or_default();
        let selection_color = gpui::hsla(focus_color.h, focus_color.s, focus_color.l, 0.25);

        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let placeholder_for_element = state.read(cx).placeholder.clone();
        let inner = TextAreaElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color,
            cursor_color,
            selection_color,
            placeholder: placeholder_for_element,
            min_h,
        };

        let base: Stateful<gpui::Div> = div()
            .id(id.clone())
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .p(padding.top)
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .child(inner);

        let focused_div: Stateful<gpui::Div> = base.track_focus(&focus_handle);

        let mut keyed: Stateful<gpui::Div> = focused_div
            .key_context("UITextInput")
            .on_action(action_handler!(
                state.clone(),
                disabled,
                Backspace,
                backspace
            ))
            .on_action(action_handler!(state.clone(), disabled, Delete, delete))
            .on_action(action_handler!(state.clone(), disabled, Left, left))
            .on_action(action_handler!(state.clone(), disabled, Right, right))
            .on_action(action_handler!(
                state.clone(),
                disabled,
                SelectLeft,
                select_left
            ))
            .on_action(action_handler!(
                state.clone(),
                disabled,
                SelectRight,
                select_right
            ))
            .on_action(action_handler!(
                state.clone(),
                disabled,
                SelectAll,
                select_all
            ))
            .on_action(action_handler!(state.clone(), disabled, Home, home))
            .on_action(action_handler!(state.clone(), disabled, End, end))
            .on_action(action_handler!(
                state.clone(),
                disabled,
                ShowCharacterPalette,
                show_character_palette
            ))
            .on_action(action_handler!(state.clone(), disabled, Paste, paste))
            .on_action(action_handler!(state.clone(), disabled, Cut, cut))
            .on_action(action_handler!(state.clone(), disabled, Copy, copy));

        let state_for_enter = state.clone();
        let on_change_for_enter = on_change.clone();
        keyed = keyed.on_action(move |_: &Enter, window, cx| {
            if disabled {
                return;
            }
            let before = state_for_enter.read(cx).value.clone();
            state_for_enter.update(cx, |s, _cx| {
                s.insert_text("\n");
            });
            let after = state_for_enter.read(cx).value.clone();
            if before != after
                && let Some(cb) = on_change_for_enter.as_ref()
            {
                cb(&after, window, &mut *cx);
            }
        });

        let state_for_mouse = state.clone();
        let state_for_move = state.clone();
        let state_for_up = state.clone();
        keyed = keyed
            .on_mouse_down(
                MouseButton::Left,
                move |event: &MouseDownEvent, window, cx| {
                    state_for_mouse.update(cx, |s, cx| {
                        s.on_mouse_down(event.position, window, cx);
                    });
                },
            )
            .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                state_for_up.update(cx, |s, cx| s.on_mouse_up(event, window, cx));
            })
            .on_mouse_move(move |event: &MouseMoveEvent, window, cx| {
                state_for_move.update(cx, |s, cx| s.on_mouse_move(event, window, cx));
            });

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);
        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .into_any_element()
    }
}
