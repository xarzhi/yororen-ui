//! `PasswordInputRenderer` — visual side of `PasswordInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` (the inner
//! painter) but the renderer shows the `mask_char` repeated for
//! the value's char count instead of the raw value. The real
//! value lives in `TextInputState.value`; only the *display* is
//! masked.

use std::any::Any;
use std::sync::Arc;

use gpui::{
    div, px, AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    Stateful, StatefulInteractiveElement, Styled, Window,
};
use yororen_ui_core::headless::password_input::PasswordInputProps;
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::spec::Edges;
use crate::renderers::text_input::{
    start_cursor_blink, wire_input_keyboard, TextInputElement, TextInputRenderState,
    TextInputRenderer,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct PasswordInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait PasswordInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenPasswordInputRenderer;

impl PasswordInputRenderer for TokenPasswordInputRenderer {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else if state.has_custom_bg {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else if state.has_custom_border {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        } else {
            theme.get_color("border.default").unwrap_or_default()
        }
    }
    fn focus_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_focus_border {
            state
                .custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        }
    }
    fn hover_border(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else {
            state
                .custom_fg
                .unwrap_or_else(|| theme.get_color("content.primary").unwrap_or_default())
        }
    }
    fn min_height(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.input.min_height").unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme.get_number("tokens.control.input.horizontal_padding").unwrap_or(0.0) as f32),
            px(theme.get_number("tokens.control.input.vertical_padding").unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_password_input<T: PasswordInputRenderer + 'static>(r: T) -> Arc<dyn PasswordInputRenderer> {
    Arc::new(r)
}

pub trait DefaultPasswordInput: Sized {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement;
}

impl DefaultPasswordInput for PasswordInputProps {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement {
        let theme_arc = cx.theme().clone();
                let r: Arc<dyn PasswordInputRenderer> = cx
            .renderer_arc::<markers::PasswordInput, dyn PasswordInputRenderer>()
            .expect("PasswordInputRenderer registered").clone();
        let theme = &*theme_arc;

        let id = self.id.clone();
        let placeholder_str = self.placeholder.clone();
        let disabled = self.disabled;
        let max_length = self.max_length;
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let mask_char = self.mask_char;

        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(placeholder_str);
            s.max_length = max_length;
            s.on_change = on_change.clone();
            s.on_submit = on_submit.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);

        let render_state = PasswordInputRenderState {
            disabled,
            focused,
            has_custom_bg: self.has_custom_bg,
            has_custom_border: self.has_custom_border,
            has_custom_focus_border: self.has_custom_focus_border,
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

        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        // Compute masked display value once.
        let value_len = state.read(cx).value.chars().count();
        let masked: String = std::iter::repeat(mask_char).take(value_len).collect();

        // The inner element: we use the *masked* string as the
        // value, but we also push the real value into the state
        // for the IME / action pipeline to read.
        // To make this work without diverging, we set the state's
        // value placeholder field to the mask and let the
        // TextInputElement shape the masked line. (The state.value
        // is still the real text — the element just displays the
        // placeholder-text-equivalent of the mask.)
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(masked);
        });

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color: text_color,
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
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                gpui::CursorStyle::Arrow
            } else {
                gpui::CursorStyle::IBeam
            });

        let focused_div: Stateful<Div> = base.track_focus(&focus_handle);
        let keyed = wire_input_keyboard(
            focused_div,
            state.clone(),
            focus_handle.clone(),
            disabled,
            on_submit,
        );

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);
        let final_div = keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(inner);

        final_div.into_any_element()
    }
}
