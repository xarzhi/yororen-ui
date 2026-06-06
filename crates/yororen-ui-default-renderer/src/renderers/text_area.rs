//! `TextAreaRenderer` — visual side of `TextArea`.
//!
//! v0.3 implementation: reuses `TextInputElement` (the inner
//! painter). Enter inserts a newline instead of firing
//! on_submit. The on_action(Enter) handler is overridden to
//! call `state.update` with `s.insert_text("\n")` instead of
//! the default Enter behaviour.

use std::any::Any;
use std::sync::Arc;

use gpui::{
    div, px, AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window,
};
use yororen_ui_core::headless::text_area::TextAreaProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::spec::Edges;
use crate::renderers::text_input::{
    start_cursor_blink, TextInputElement, TextInputRenderState, TextInputRenderer,
};
use yororen_ui_core::action_handler;
use yororen_ui_core::headless::text_input::{
    Backspace, Copy, Cut, Delete, End, Enter, Home, Left, Paste, Right, SelectAll, SelectLeft,
    SelectRight, ShowCharacterPalette,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct TextAreaRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextAreaRenderer: Any + Send + Sync {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TextAreaRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &TextAreaRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenTextAreaRenderer;

impl TextAreaRenderer for TokenTextAreaRenderer {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    fn focus_border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    fn hover_border(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else {
            state
                .custom_text_color
                .unwrap_or_else(|| theme.get_color("content.primary").unwrap_or_default())
        }
    }
    fn min_height(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.input.text_area_min_h").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(px(
            theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32,
        ))
    }
    fn border_radius(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_text_area<T: TextAreaRenderer + 'static>(r: T) -> Arc<dyn TextAreaRenderer> {
    Arc::new(r)
}

pub trait DefaultTextArea: Sized {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement;
}

impl DefaultTextArea for TextAreaProps {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement {
        let theme_arc = cx.theme().clone();
                let r: Arc<dyn TextAreaRenderer> = cx
            .renderer_arc::<markers::TextArea, dyn TextAreaRenderer>()
            .expect("TextAreaRenderer registered").clone();
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
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);

        let render_state = TextAreaRenderState {
            disabled,
            focused,
            has_custom_bg: self.has_custom_bg,
            custom_bg: self.custom_bg,
            custom_border: self.custom_border,
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
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);

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
        };

        let base: Stateful<Div> = div()
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
                gpui::CursorStyle::Arrow
            } else {
                gpui::CursorStyle::IBeam
            });

        let focused_div: Stateful<Div> = base.track_focus(&focus_handle);

        // Wire the 14 standard on_action handlers, but override
        // Enter to insert a newline (multi-line).
        let mut keyed: Stateful<Div> = focused_div
            .key_context("UITextInput")
            .on_action(action_handler!(state.clone(), disabled, Backspace, backspace))
            .on_action(action_handler!(state.clone(), disabled, Delete, delete))
            .on_action(action_handler!(state.clone(), disabled, Left, left))
            .on_action(action_handler!(state.clone(), disabled, Right, right))
            .on_action(action_handler!(state.clone(), disabled, SelectLeft, select_left))
            .on_action(action_handler!(state.clone(), disabled, SelectRight, select_right))
            .on_action(action_handler!(state.clone(), disabled, SelectAll, select_all))
            .on_action(action_handler!(state.clone(), disabled, Home, home))
            .on_action(action_handler!(state.clone(), disabled, End, end))
            .on_action(action_handler!(state.clone(), disabled, ShowCharacterPalette, show_character_palette))
            .on_action(action_handler!(state.clone(), disabled, Paste, paste))
            .on_action(action_handler!(state.clone(), disabled, Cut, cut))
            .on_action(action_handler!(state.clone(), disabled, Copy, copy));

        // Enter inserts '\n' (multi-line behaviour). Fires
        // on_change if the value actually changes.
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

        // Mouse handlers (focus on click).
        let state_for_mouse = state.clone();
        keyed = keyed
            .on_mouse_down(gpui::MouseButton::Left, move |event: &gpui::MouseDownEvent, window, cx| {
                state_for_mouse.update(cx, |s, cx| {
                    s.on_mouse_down(event.position, window, cx);
                });
            })
            .on_mouse_move(move |event: &gpui::MouseMoveEvent, window, cx| {
                state.update(cx, |s, cx| s.on_mouse_move(event, window, cx));
            });

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);
        let final_div = keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(inner);

        final_div.into_any_element()
    }
}
