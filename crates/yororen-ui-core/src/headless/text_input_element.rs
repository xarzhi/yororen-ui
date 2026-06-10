//! `TextInputElement` — the inner painter for any single-line
//! text-input headless component.
//!
//! Lives in headless (not the renderer) because it carries no
//! theme knowledge: it shapes the line, paints the selection quad
//! and caret, and registers the IME pipeline against the
//! `TextInputState`. The colours are passed in by the caller
//! (`headless::XxxProps::render`), which reads them from the
//! registered `XxxRenderer`.

use std::time::Duration;

use gpui::{
    App, Bounds, CursorStyle, Div, Element, ElementId, ElementInputHandler, FocusHandle,
    GlobalElementId, Hsla, InteractiveElement, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PaintQuad, ParentElement, Pixels, ShapedLine, SharedString,
    Stateful, StatefulInteractiveElement, Style, Styled, TextRun, Window, fill, hsla, point, px,
    relative, size,
};

use crate::action_handler;
use crate::headless::text_input::{
    Backspace, Copy, Cut, Delete, End, Enter, Escape, Home, Left, Paste, Right, SelectAll,
    SelectLeft, SelectRight, ShowCharacterPalette, TextInputState,
};

/// How often the caret blinks while focused. 500ms matches the
/// gpui / WebKit convention.
pub const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// The inner element for any single-line text-input component.
///
/// Embedded by `headless::XxxProps::render` (for `text_input`,
/// `password_input`, `search_input`, etc.) inside the wrapper
/// `div` that supplies focus / keymap / border / padding.
pub struct TextInputElement {
    pub state: gpui::Entity<TextInputState>,
    pub focus_handle: FocusHandle,
    pub disabled: bool,
    pub text_color: Hsla,
    pub hint_color: Hsla,
    pub cursor_color: Hsla,
    pub selection_color: Hsla,
    pub placeholder: SharedString,
    /// When `Some`, used as the **display** text instead of
    /// `state.value` (for `password_input`'s mask). The state's
    /// real value still drives the caret, selection, IME and
    /// `on_change` — only the visual line is overridden. The
    /// override must have the same char-count as
    /// `state.value` so byte indices line up.
    pub value_override: Option<String>,
}

pub struct PrepaintState {
    pub line: Option<ShapedLine>,
    pub selection: Option<PaintQuad>,
    pub cursor: Option<PaintQuad>,
    pub scroll_x: Pixels,
    pub display_text: SharedString,
    pub cursor_visible: bool,
}

impl IntoElement for TextInputElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextInputElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.state.read(cx);
        let value = input.value.clone();
        let caret = input.caret;
        let selection = input.selected_range();
        let placeholder = input.placeholder.clone();
        let scroll_x_input = input.scroll_x;
        let cursor_visible = input.cursor_visible;

        let is_empty = value.is_empty();
        let (display_text, run_color) = if is_empty {
            (placeholder, self.hint_color)
        } else if let Some(override_text) = &self.value_override {
            (SharedString::from(override_text.clone()), self.text_color)
        } else {
            (SharedString::from(value.clone()), self.text_color)
        };

        let mask_char_bytes = self
            .value_override
            .as_ref()
            .and_then(|s| s.chars().next())
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        let (caret_disp, selection_disp) = if self.value_override.is_some() {
            let real_value_caret = caret.min(value.len());
            let caret_chars = value[..real_value_caret].chars().count();
            let start_chars = value[..selection.start.min(value.len())].chars().count();
            let end_chars = value[..selection.end.min(value.len())].chars().count();
            (
                caret_chars * mask_char_bytes,
                (start_chars * mask_char_bytes)..(end_chars * mask_char_bytes),
            )
        } else {
            (caret, selection)
        };

        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: run_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window
            .text_system()
            .shape_line(display_text.clone(), font_size, &[run], None);

        let caret_x_line = line.x_for_index(caret_disp);
        let cursor_thickness = px(2.0);
        let max_cursor_x = (bounds.size.width - cursor_thickness).max(Pixels::ZERO);
        let max_scroll_x = (line.width - max_cursor_x).max(Pixels::ZERO);
        let mut scroll_x = scroll_x_input.clamp(Pixels::ZERO, max_scroll_x);
        if caret_x_line < scroll_x {
            scroll_x = caret_x_line;
        } else if caret_x_line > scroll_x + max_cursor_x {
            scroll_x = caret_x_line - max_cursor_x;
        }
        scroll_x = scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        let (selection_quad, cursor_quad) = if !selection_disp.is_empty() && !is_empty {
            let start_x = line.x_for_index(selection_disp.start);
            let end_x = line.x_for_index(selection_disp.end);
            let quad = fill(
                Bounds::from_corners(
                    point(bounds.left() + start_x.min(end_x) - scroll_x, bounds.top()),
                    point(
                        bounds.left() + start_x.max(end_x) - scroll_x,
                        bounds.bottom(),
                    ),
                ),
                self.selection_color,
            );
            (Some(quad), None)
        } else {
            let cursor_paint_x = bounds.left() + caret_x_line - scroll_x;
            let quad = fill(
                Bounds::new(
                    point(cursor_paint_x, bounds.top()),
                    size(cursor_thickness, bounds.bottom() - bounds.top()),
                ),
                self.cursor_color,
            );
            (None, Some(quad))
        };

        PrepaintState {
            line: Some(line),
            selection: selection_quad,
            cursor: cursor_quad,
            scroll_x,
            display_text,
            cursor_visible,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if !self.disabled {
            window.handle_input(
                &self.focus_handle,
                ElementInputHandler::new(bounds, self.state.clone()),
                cx,
            );
        }

        if let Some(sel) = prepaint.selection.take() {
            window.paint_quad(sel);
        }

        let line = prepaint
            .line
            .take()
            .expect("prepaint always produces a line");
        let origin_x = bounds.left() - prepaint.scroll_x;
        let _ = line.paint(
            point(origin_x, bounds.top()),
            window.line_height(),
            window,
            cx,
        );

        let is_focused = self.focus_handle.is_focused(window);
        if is_focused
            && prepaint.cursor_visible
            && let Some(cur) = prepaint.cursor.take()
        {
            window.paint_quad(cur);
        }

        self.state.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            input.scroll_x = prepaint.scroll_x;
        });
    }
}

/// The submit callback (Enter / on_submit) shared by every input
/// component. Lives in headless because the action handler chain
/// references it directly.
pub type SubmitCallback = std::sync::Arc<dyn Fn(&str, &mut Window, &mut App) + Send + Sync>;

/// Apply the text-input keymap (`track_focus` + `key_context` +
/// 14 `on_action` handlers + 3 mouse handlers) to `wrapper`.
///
/// The wrapper's `.id(...)` is NOT applied here — the caller is
/// expected to set it before calling, and the `key_context`
/// assumes the focus is in the right scope.
pub fn wire_input_keyboard(
    mut wrapper: Stateful<Div>,
    state: gpui::Entity<TextInputState>,
    _focus_handle: FocusHandle,
    disabled: bool,
    on_submit: Option<SubmitCallback>,
) -> Stateful<Div> {
    wrapper = wrapper
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
    let on_submit_for_enter = on_submit.clone();
    wrapper = wrapper.on_action(move |_: &Enter, window, cx| {
        if disabled {
            return;
        }
        let value = state_for_enter.read(cx).value.clone();
        if let Some(cb) = on_submit_for_enter.as_ref() {
            cb(&value, window, cx);
        }
    });

    let state_for_mouse = state.clone();
    let state_for_up = state.clone();
    let state_for_up_out = state.clone();
    let state_for_move = state.clone();
    wrapper = wrapper
        .on_mouse_down(
            MouseButton::Left,
            move |event: &MouseDownEvent, window, cx| {
                state_for_mouse.update(cx, |s, cx| {
                    s.on_mouse_down(event.position, window, cx);
                });
            },
        )
        .on_mouse_up(
            MouseButton::Left,
            move |event: &MouseUpEvent, window, cx| {
                state_for_up.update(cx, |s, cx| s.on_mouse_up(event, window, cx));
            },
        )
        .on_mouse_up_out(
            MouseButton::Left,
            move |event: &MouseUpEvent, window, cx| {
                state_for_up_out.update(cx, |s, cx| s.on_mouse_up(event, window, cx));
            },
        )
        .on_mouse_move(move |event: &MouseMoveEvent, window, cx| {
            state_for_move.update(cx, |s, cx| s.on_mouse_move(event, window, cx));
        });

    wrapper
}

/// Start the cursor-blink task. The state has a
/// `cursor_blink_epoch` counter; the running task checks it on
/// each tick and exits if it changed (i.e. focus moved
/// elsewhere).
pub fn start_cursor_blink(state: gpui::Entity<TextInputState>, window: &mut Window, cx: &mut App) {
    state.update(cx, |s, _cx| {
        s.cursor_blink_epoch = s.cursor_blink_epoch.wrapping_add(1);
    });
    let epoch = state.read(cx).cursor_blink_epoch;
    let state_weak = state.downgrade();
    window
        .spawn(cx, async move |async_cx| {
            loop {
                async_cx
                    .background_executor()
                    .timer(CURSOR_BLINK_INTERVAL)
                    .await;
                let should_continue = async_cx
                    .update(|window, cx| {
                        state_weak
                            .update(cx, |s, cx| {
                                if s.cursor_blink_epoch != epoch {
                                    s.cursor_visible = true;
                                    cx.notify();
                                    return false;
                                }
                                if !s.focus_handle().is_focused(window) {
                                    s.cursor_visible = true;
                                    cx.notify();
                                    return false;
                                }
                                s.cursor_visible = !s.cursor_visible;
                                cx.notify();
                                true
                            })
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);
                if !should_continue {
                    return;
                }
            }
        })
        .detach();
}

// `hsla` re-export so callers don't have to import gpui::hsla
// just to build a `HintColor` or `CursorColor`. Kept here so the
// headless text-input element stays renderer-agnostic.
#[allow(dead_code)]
pub(crate) fn hsla_default() -> Hsla {
    hsla(0.0, 0.0, 0.0, 1.0)
}
