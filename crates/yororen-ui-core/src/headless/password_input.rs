//! Headless `password_input` — text input that masks the value.
//!
//! Reuses `TextInputState` (the renderer mints it via
//! `use_keyed_state`); the visual masks the value with `mask_char`.

use std::sync::Arc;

use gpui::{App, Hsla};

#[derive(Clone)]
pub struct PasswordInputProps {
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<super::text_input::TextChangeCallback>,
    pub on_submit: Option<super::text_input::TextChangeCallback>,
    /// Character to display for each typed letter. Defaults
    /// to `•` (U+2022).
    pub mask_char: char,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn password_input(id: impl Into<gpui::ElementId>) -> PasswordInputProps {
    PasswordInputProps {
        id: id.into(),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        on_submit: None,
        mask_char: '•',
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl PasswordInputProps {
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
    pub fn mask_char(mut self, c: char) -> Self {
        self.mask_char = c;
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

    /// Render the password input using the registered `PasswordInputRenderer`.
    pub fn render(self, cx: &mut gpui::App, window: &mut gpui::Window) -> gpui::AnyElement {
        use crate::headless::text_input::TextInputState;
        use crate::headless::text_input_element::{
            TextInputElement, start_cursor_blink, wire_input_keyboard,
        };
        use crate::renderer::RendererContext;
        use crate::renderer::markers::PasswordInput as PasswordInputMarker;
        use crate::renderer::password_input::{PasswordInputRenderState, PasswordInputRenderer};
        use crate::renderer::spec::Edges;
        use crate::theme::ActiveTheme;
        use gpui::{
            CursorStyle, InteractiveElement, IntoElement, ParentElement, Stateful,
            StatefulInteractiveElement, Styled, div,
        };
        use std::sync::Arc;

        let theme_arc = cx.theme().clone();
        let r: Arc<dyn PasswordInputRenderer> = cx
            .renderer_arc::<PasswordInputMarker, dyn PasswordInputRenderer>()
            .expect("PasswordInputRenderer registered")
            .clone();
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
        let padding: Edges<gpui::Pixels> = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);

        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let value_len = state.read(cx).value.chars().count();
        let masked: String = std::iter::repeat_n(mask_char, value_len).collect();

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color: theme.get_color("content.tertiary").unwrap_or_default(),
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
            value_override: Some(masked),
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
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .child(inner);

        let focused_div: Stateful<gpui::Div> = base.track_focus(&focus_handle);
        let keyed = wire_input_keyboard(
            focused_div,
            state.clone(),
            focus_handle.clone(),
            disabled,
            on_submit,
        );

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);
        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .into_any_element()
    }
}
