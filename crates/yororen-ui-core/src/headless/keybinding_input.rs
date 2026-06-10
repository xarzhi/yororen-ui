//! Headless `keybinding_input` — text input that captures
//! keystrokes (instead of typing them) when in capture mode.

use std::sync::Arc;

use gpui::{App, Hsla};

pub type KeybindingChangeCallback = Arc<dyn Fn(&str, &mut gpui::Window, &mut App) + Send + Sync>;
pub type KeybindingCaptureCallback = Arc<dyn Fn(&mut gpui::Window, &mut App) + Send + Sync>;

/// `KeybindingInput` is in one of two states: idle (user is
/// typing) or capturing (next keystroke is recorded as a
/// keybinding combo).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeybindingInputMode {
    #[default]
    Idle,
    Capturing,
}

#[derive(Clone)]
pub struct KeybindingInputProps {
    pub id: gpui::ElementId,
    pub mode: KeybindingInputMode,
    pub disabled: bool,
    pub placeholder: String,
    pub on_change: Option<KeybindingChangeCallback>,
    pub on_start_capture: Option<KeybindingCaptureCallback>,
    pub on_cancel_capture: Option<KeybindingCaptureCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn keybinding_input(id: impl Into<gpui::ElementId>) -> KeybindingInputProps {
    KeybindingInputProps {
        id: id.into(),
        mode: KeybindingInputMode::Idle,
        disabled: false,
        placeholder: "Press a key combo…".to_string(),
        on_change: None,
        on_start_capture: None,
        on_cancel_capture: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl KeybindingInputProps {
    pub fn mode(mut self, m: KeybindingInputMode) -> Self {
        self.mode = m;
        self
    }
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_start_capture<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut gpui::Window, &mut App),
    {
        self.on_start_capture = Some(Arc::new(f));
        self
    }
    pub fn on_cancel_capture<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&mut gpui::Window, &mut App),
    {
        self.on_cancel_capture = Some(Arc::new(f));
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

    /// Render the keybinding input using the registered `KeybindingInputRenderer`.
    pub fn render(self, cx: &mut gpui::App, window: &mut gpui::Window) -> gpui::AnyElement {
        use crate::headless::keybinding_input::KeybindingInputMode;
        use crate::headless::text_input::TextInputState;
        use crate::headless::text_input_element::{
            TextInputElement, start_cursor_blink, wire_input_keyboard,
        };
        use crate::renderer::RendererContext;
        use crate::renderer::keybinding_input::{
            KeybindingInputRenderState, KeybindingInputRenderer,
        };
        use crate::renderer::markers::KeybindingInput as KeybindingInputMarker;
        use crate::theme::ActiveTheme;
        use gpui::{
            CursorStyle, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement,
            Stateful, StatefulInteractiveElement, Styled, div, px,
        };
        use std::sync::Arc;

        let theme_arc = cx.theme().clone();
        let r: Arc<dyn KeybindingInputRenderer> = cx
            .renderer_arc::<KeybindingInputMarker, dyn KeybindingInputRenderer>()
            .expect("KeybindingInputRenderer registered")
            .clone();
        let theme = &*theme_arc;

        let id = self.id.clone();
        let placeholder_str = self.placeholder.clone();
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let on_start_capture = self.on_start_capture.clone();
        let on_cancel_capture = self.on_cancel_capture.clone();
        let mode = self.mode;

        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(placeholder_str);
            s.on_change = on_change.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);

        let render_state = KeybindingInputRenderState {
            capturing: mode == KeybindingInputMode::Capturing,
            disabled,
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
        let kbd_bg = r.kbd_bg(&render_state, theme);
        let kbd_fg = r.kbd_fg(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
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
            text_color: kbd_fg,
            hint_color: theme.get_color("content.tertiary").unwrap_or_default(),
            cursor_color: kbd_fg,
            selection_color: kbd_fg,
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
            .px(px(10.))
            .py(px(4.))
            .flex()
            .items_center()
            .text_color(kbd_fg)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            });

        let focused_div: Stateful<gpui::Div> = base.track_focus(&focus_handle);

        let mut keyed: Stateful<gpui::Div> = wire_input_keyboard(
            focused_div,
            state.clone(),
            focus_handle.clone(),
            disabled,
            None,
        );

        if mode == KeybindingInputMode::Capturing && !disabled {
            let state_for_capture = state.clone();
            let on_change_for_capture = on_change.clone();
            let on_cancel_for_capture = on_cancel_capture.clone();
            keyed = keyed.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let ks = &ev.keystroke;
                if ks.key.as_str() == "escape" {
                    if let Some(cb) = on_cancel_for_capture.as_ref() {
                        cb(window, cx);
                    }
                    return;
                }
                let mut parts: Vec<String> = Vec::new();
                if ks.modifiers.control {
                    parts.push("ctrl".into());
                }
                if ks.modifiers.alt {
                    parts.push("alt".into());
                }
                if ks.modifiers.shift {
                    parts.push("shift".into());
                }
                if ks.modifiers.platform {
                    parts.push("cmd".into());
                }
                if ks.key.is_empty() {
                    return;
                }
                parts.push(ks.key.clone());
                let combo = parts.join("-");
                state_for_capture.update(cx, |s, _cx| {
                    s.value = combo.clone();
                    s.caret = combo.len();
                    s.selection_start = combo.len();
                    s.selection_end = combo.len();
                });
                if let Some(cb) = on_change_for_capture.as_ref() {
                    cb(&combo, window, cx);
                }
            });
        }

        let on_start_clone = on_start_capture.clone();
        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);
        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                if let Some(cb) = on_start_clone.as_ref() {
                    cb(window, cx);
                }
            })
            .child(
                div()
                    .bg(kbd_bg)
                    .rounded(px(4.0))
                    .px(px(8.))
                    .py(px(2.))
                    .text_color(kbd_fg)
                    .child(if mode == KeybindingInputMode::Capturing {
                        if state.read(cx).value.is_empty() {
                            "Press a key…".to_string()
                        } else {
                            state.read(cx).value.clone()
                        }
                    } else if state.read(cx).value.is_empty() {
                        "(unset)".to_string()
                    } else {
                        state.read(cx).value.clone()
                    }),
            )
            .into_any_element()
    }
}
