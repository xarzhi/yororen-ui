//! `KeybindingInputRenderer` — visual side of `KeybindingInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` (the inner
//! painter). In `Idle` mode the wrapper has the standard
//! `key_context("UITextInput")` + 14 on_action handlers and
//! acts like a regular text input. In `Capturing` mode the
//! wrapper does NOT register the IME handler (so typing doesn't
//! insert text); instead, the next keystroke is captured as a
//! keybinding combo string (e.g. "ctrl-shift-p") and written
//! to the state value. Escape cancels capture.

use std::any::Any;
use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, Div, Hsla, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window, div,
    px,
};
use yororen_ui_core::headless::keybinding_input::{KeybindingInputMode, KeybindingInputProps};
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::text_input::{TextInputElement, start_cursor_blink, wire_input_keyboard};
pub use yororen_ui_core::renderer::keybinding_input::{
    KeybindingInputRenderState, KeybindingInputRenderer,
};

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
    fn hover_border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn kbd_bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    fn kbd_fg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    fn min_height(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.input.min_height")
            .unwrap_or(0.0) as f32)
    }
    fn border_radius(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_keybinding_input<T: KeybindingInputRenderer + 'static>(
    r: T,
) -> Arc<dyn KeybindingInputRenderer> {
    Arc::new(r)
}
