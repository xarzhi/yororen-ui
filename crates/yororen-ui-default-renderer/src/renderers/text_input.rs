//! `TextInputRenderer` — visual side of `TextInput`.
//!
//! v0.3 implementation follows the v0.2 pattern: the renderer
//! mints a `TextInputState` via `window.use_keyed_state`, the
//! wrapper `div` carries `track_focus` + `key_context` + 14
//! `.on_action(...)` handlers, and the inner `TextInputElement`
//! is a custom `gpui::Element` that shapes the text, paints the
//! selection quad, paints the line, and calls
//! `window.handle_input(&focus_handle, ElementInputHandler::new(bounds, state.clone()), cx)`
//! in `paint` to register the IME / clipboard pipeline.

use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyElement, App, Bounds, CursorStyle, Div, Element, ElementId, ElementInputHandler,
    FocusHandle, GlobalElementId, Hsla, InteractiveElement, IntoElement, LayoutId, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, ParentElement, Pixels, ShapedLine,
    SharedString, Stateful, StatefulInteractiveElement, Style, Styled, TextRun, Window, div, fill,
    hsla, point, px, relative, size,
};
use yororen_ui_core::action_handler;
use yororen_ui_core::headless::text_input::{
    Backspace, Copy, Cut, Delete, End, Enter, Home, Left, Paste, Right, SelectAll, SelectLeft,
    SelectRight, ShowCharacterPalette, TextInputProps, TextInputState,
};
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub(crate) type SubmitCallback = Arc<dyn Fn(&str, &mut Window, &mut App) + Send + Sync>;

use crate::renderers::spec::Edges;

// =====================================================================
// `TextInputRenderer` trait — visual contract (bg / border / colors /
// spacing). Unchanged from the v0.3 simplified version.
// =====================================================================

#[derive(Clone, Copy, Debug, Default)]
pub struct TextInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub trait TextInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn hint_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn cursor_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn selection_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &TextInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &TextInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &TextInputRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &TextInputRenderState, theme: &Theme) -> f32;
}

pub struct TokenTextInputRenderer;

impl TextInputRenderer for TokenTextInputRenderer {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
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
    fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
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
    fn focus_border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_focus_border {
            state
                .custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        }
    }
    fn hover_border(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.custom_text_color.is_some() {
            state.custom_text_color.unwrap()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    fn hint_color(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    fn cursor_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_focus_border {
            state
                .custom_focus_border
                .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
        } else {
            theme.get_color("border.focus").unwrap_or_default()
        }
    }
    fn selection_color(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        let c = theme.get_color("border.focus").unwrap_or_default();
        hsla(c.h, c.s, c.l, 0.25)
    }
    fn min_height(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.input.min_height")
            .unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &TextInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    fn border_radius(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn disabled_opacity(&self, state: &TextInputRenderState, _theme: &Theme) -> f32 {
        if state.disabled { 0.6 } else { 1.0 }
    }
}

pub fn arc_text_input<T: TextInputRenderer + 'static>(r: T) -> Arc<dyn TextInputRenderer> {
    Arc::new(r)
}

// =====================================================================
// `TextInputElement` — the inner `gpui::Element` that shapes the
// text, paints selection / line / cursor, and registers the IME
// handler. Mirrors v0.2's `TextLineElement`. **Public** so the
// derived input renderers (search / password / number / file_path
// / keybinding) can embed it inside their own wrapper divs.
// =====================================================================

/// How often the caret blinks while focused. 500ms matches the
/// gpui / WebKit convention.
pub const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// The inner element for any single-line text-input component.
/// Public so the derived renderers can embed it.
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

        // Pick the display text. If `value_override` is set
        // (e.g. `password_input`'s mask), use it instead of
        // `value` for the visual line — the state's real value
        // still drives the caret / selection / IME / on_change.
        let is_empty = value.is_empty();
        let (display_text, run_color) = if is_empty {
            (placeholder, self.hint_color)
        } else if let Some(override_text) = &self.value_override {
            (SharedString::from(override_text.clone()), self.text_color)
        } else {
            (SharedString::from(value.clone()), self.text_color)
        };

        // Map the real-value caret (in UTF-8 bytes) to the
        // display-text byte position. For `value_override`
        // (the password mask), the display is `mask_char`
        // repeated N times where N = `value.chars().count()`.
        // **Important**: the mask char itself may be multi-byte
        // (e.g. `•` is 3 bytes, `●` is 3 bytes) — so the
        // display byte index for real-value char `n` is
        // `n * mask_char.len_utf8()`, NOT just `n`.
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

// =====================================================================
// `wire_input_keyboard` — shared helper. The 14 on_action handlers
// are identical for all 7 input components; this function applies
// them to a wrapper div. Plus `track_focus` and `key_context`.
// =====================================================================

/// Apply the text-input keymap (track_focus + key_context +
/// 14 on_action handlers + 3 mouse handlers) to `wrapper`.
///
/// The wrapper's `.id(...)` is NOT applied here — the caller is
/// expected to set it before calling, and the `key_context`
/// assumes the focus is in the right scope.
pub fn wire_input_keyboard(
    mut wrapper: Stateful<Div>,
    state: gpui::Entity<TextInputState>,
    focus_handle: FocusHandle,
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

    // Enter: fire on_submit. This isn't an action_handler! because
    // we also need the on_submit callback (which is owned by the
    // renderer's `props`, not the state).
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

    // Mouse handlers — focus on click, drag-select while held.
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

    // Blur on click-outside.
    let _ = focus_handle; // reserved for future focus_handle-aware wiring
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

// =====================================================================
// `DefaultTextInput` — `headless::TextInputProps` sugar. Returns
// `AnyElement` (the v0.2 pattern).
// =====================================================================

pub trait DefaultTextInput: Sized {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement;
}

impl DefaultTextInput for TextInputProps {
    fn default_render(self, cx: &mut App, window: &mut Window) -> AnyElement {
        let theme_arc = cx.theme().clone();
        let r: Arc<dyn TextInputRenderer> = cx
            .renderer_arc::<markers::TextInput, dyn TextInputRenderer>()
            .expect("TextInputRenderer registered")
            .clone();
        let theme = &*theme_arc;

        // Mint the state.
        let id = self.id.clone();
        let placeholder_str = self.placeholder.clone();
        let max_length = self.max_length;
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();

        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        // Mirror props into state.
        state.update(cx, |s, _cx| {
            s.placeholder = SharedString::from(placeholder_str);
            s.max_length = max_length;
            s.on_change = on_change;
            s.on_submit = on_submit.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);

        // Resolve the visual.
        let render_state = TextInputRenderState {
            disabled,
            focused,
            has_custom_bg: self.has_custom_bg,
            has_custom_border: self.has_custom_border,
            has_custom_focus_border: self.has_custom_focus_border,
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
        let hint_color = r.hint_color(&render_state, theme);
        let cursor_color = r.cursor_color(&render_state, theme);
        let selection_color = r.selection_color(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let opacity = r.disabled_opacity(&render_state, theme);

        let placeholder_for_element = state.read(cx).placeholder.clone();

        // Cursor blink.
        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        // The inner painter.
        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color,
            cursor_color,
            selection_color,
            placeholder: placeholder_for_element,
            value_override: None,
        };

        // Build the wrapper div. The chain is:
        //   Div (with styling + id + child) -> Stateful<Div>
        //   (track_focus + keymap + mouse + hover/active).
        // `track_focus` returns `Stateful<Div>` which does NOT
        // implement `ParentElement`, so we must add the inner
        // child BEFORE `track_focus`.
        let base: Stateful<Div> = div()
            .id(id.clone())
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .opacity(opacity)
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

        let focused_div: Stateful<Div> = base.track_focus(&focus_handle);
        let keyed: Stateful<Div> = wire_input_keyboard(
            focused_div,
            state.clone(),
            focus_handle.clone(),
            disabled,
            on_submit,
        );

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);
        let final_div: Stateful<Div> = keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border));

        final_div.into_any_element()
    }
}
