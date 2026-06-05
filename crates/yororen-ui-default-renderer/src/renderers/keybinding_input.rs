//! `KeybindingInputRenderer` — visual side of `KeybindingInput`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use crate::renderers::spec::Edges;
use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct KeybindingInputRenderState {
    pub capturing: bool,
    pub disabled: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait KeybindingInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn kbd_bg(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn kbd_fg(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Hsla;
    fn kbd_padding(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn kbd_min_width(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
    fn min_height(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
    fn icon_size(&self, state: &KeybindingInputRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenKeybindingInputRenderer;

impl KeybindingInputRenderer for TokenKeybindingInputRenderer {
    fn bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn focus_border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn kbd_bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    fn kbd_fg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    fn kbd_padding(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.control.keybinding_input.kbd_padding_x").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.control.keybinding_input.kbd_padding_y").unwrap_or(0.0) as f32),
        )
    }
    fn kbd_min_width(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.keybinding_input.kbd_min_width").unwrap_or(0.0) as f32)
    }
    fn min_height(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.input.min_height").unwrap_or(0.0) as f32)
    }
    fn border_radius(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn icon_size(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.keybinding_input.icon_size").unwrap_or(0.0) as f32)
    }
}

pub fn arc_keybinding_input<T: KeybindingInputRenderer + 'static>(
    r: T,
) -> Arc<dyn KeybindingInputRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultKeybindingInput` — `headless::KeybindingInputProps` sugar.
// =====================================================================

use gpui::{
    div, App, InteractiveElement, KeyDownEvent, MouseButton, ParentElement, Stateful, Styled,
    Window,
};
use yororen_ui_core::headless::keybinding_input::{KeybindingInputMode, KeybindingInputProps};
use yororen_ui_core::renderer::RendererContext;
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultKeybindingInput: Sized {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div>;
}

impl DefaultKeybindingInput for KeybindingInputProps {
    fn default_render(self, cx: &App, window: &Window) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn KeybindingInputRenderer> = cx
            .renderer_arc::<yororen_ui_core::renderer::markers::KeybindingInput, dyn KeybindingInputRenderer>(
            )
            .expect("KeybindingInputRenderer registered");

        let state = self.state.clone();
        let focus_handle = self.focus_handle.clone();
        let on_change = self.on_change.clone();
        let on_start_capture = self.on_start_capture.clone();
        let on_cancel_capture = self.on_cancel_capture.clone();
        let disabled = self.disabled;
        let mode = self.mode;
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
        let min_h = r.min_height(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let kbd_bg = r.kbd_bg(&render_state, theme);
        let kbd_fg = r.kbd_fg(&render_state, theme);
        let kbd_padding = r.kbd_padding(&render_state, theme);
        let kbd_min_w = r.kbd_min_width(&render_state, theme);

        let value = state.read(cx).value.clone();
        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .px(kbd_padding.left)
            .py(kbd_padding.top)
            .flex()
            .items_center()
            .text_color(kbd_fg);

        if value.is_empty() {
            el = el.child(
                div()
                    .min_w(kbd_min_w)
                    .bg(kbd_bg)
                    .rounded(px(4.0))
                    .px(kbd_padding.left)
                    .py(kbd_padding.top)
                    .text_color(kbd_fg)
                    .child(if mode == KeybindingInputMode::Capturing {
                        "Press a key…".to_string()
                    } else {
                        "(unset)".to_string()
                    }),
            );
        } else {
            el = el.child(
                div()
                    .min_w(kbd_min_w)
                    .bg(kbd_bg)
                    .rounded(px(4.0))
                    .child(value),
            );
        }

        // Click starts capture mode (idle → capturing).
        let focus_for_mouse = focus_handle.clone();
        let on_start_capture_clone = on_start_capture.clone();
        el = el.on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
            focus_for_mouse.focus(window);
            if let Some(cb) = on_start_capture_clone.as_ref() {
                cb(window, cx);
            }
        });

        if mode == KeybindingInputMode::Capturing && !disabled {
            // While capturing, intercept the next keystroke and
            // record it as the binding instead of typing it.
            let state_for_keys = state.clone();
            let on_change_for_keys = on_change.clone();
            let on_cancel_for_keys = on_cancel_capture.clone();
            el = el.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let keystroke = &ev.keystroke;
                // Escape cancels capture.
                if keystroke.key.as_str() == "escape" {
                    if let Some(cb) = on_cancel_for_keys.as_ref() {
                        cb(window, cx);
                    }
                    return;
                }
                // Build a human-readable combo string.
                let mut parts: Vec<String> = Vec::new();
                if keystroke.modifiers.control {
                    parts.push("ctrl".into());
                }
                if keystroke.modifiers.alt {
                    parts.push("alt".into());
                }
                if keystroke.modifiers.shift {
                    parts.push("shift".into());
                }
                if keystroke.modifiers.platform {
                    parts.push("cmd".into());
                }
                let key_str = if !keystroke.key.is_empty() {
                    keystroke.key.clone()
                } else {
                    return;
                };
                parts.push(key_str);
                let combo = parts.join("-");
                let new_value = state_for_keys.update(cx, |s, _cx| {
                    s.value = combo.clone();
                    s.caret = combo.len();
                    s.value.clone()
                });
                if let Some(cb) = on_change_for_keys.as_ref() {
                    cb(&new_value, window, cx);
                }
            });
        }

        self.apply(el)
    }
}

use gpui::px;
