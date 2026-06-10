//! `TextAreaRenderer` — visual side of `TextArea`.
//!
//! v0.3 multi-line implementation: a dedicated `TextAreaElement`
//! that splits the value by `'\n'`, shapes each line separately
//! (`gpui`'s `shape_line` rejects strings containing `'\n'` —
//! that's what crashed v0.2's path: a `TextInputElement` reused
//! for `text_area` happily accepted a `'\n'`-containing value and
//! then panicked in `shape_line` at the first Enter keystroke),
//! computes the caret's row + column from the byte offset, paints
//! each line at its own Y, and renders one selection quad per row
//! the selection spans. The wrapper div handles focus / key
//! dispatch / mouse (shared with the 6 other inputs via
//! `wire_input_keyboard`), with two overrides: the Enter action
//! handler inserts a newline instead of firing `on_submit`, and
//! `state.paste_newlines = true` keeps `'\n'` in pasted text
//! (single-line inputs collapse newlines to spaces).

use std::any::Any;
use std::ops::Range;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, CursorStyle, Div, Element, ElementId, ElementInputHandler,
    FocusHandle, GlobalElementId, Hsla, InteractiveElement, IntoElement, LayoutId, MouseButton,
    MouseDownEvent, MouseMoveEvent, PaintQuad, ParentElement, Pixels, ShapedLine, SharedString,
    Stateful, StatefulInteractiveElement, Style, Styled, TextRun, Window, div, fill, point, px,
    relative, size,
};
use yororen_ui_core::action_handler;
use yororen_ui_core::headless::text_area::TextAreaProps;
use yororen_ui_core::headless::text_input::{
    Backspace, Copy, Cut, Delete, End, Enter, Home, Left, Paste, Right, SelectAll, SelectLeft,
    SelectRight, ShowCharacterPalette, TextInputState,
};
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::text_input::start_cursor_blink;
use yororen_ui_core::renderer::spec::Edges;
pub use yororen_ui_core::renderer::text_area::{TextAreaRenderState, TextAreaRenderer};

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
        px(theme
            .get_number("tokens.control.input.text_area_min_h")
            .unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(px(theme
            .get_number("tokens.control.input.vertical_padding")
            .unwrap_or(0.0) as f32))
    }
    fn border_radius(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

pub fn arc_text_area<T: TextAreaRenderer + 'static>(r: T) -> Arc<dyn TextAreaRenderer> {
    Arc::new(r)
}

// =====================================================================
// `TextAreaElement` — the inner `gpui::Element` that paints a
// multi-line text area. Splits the value by `'\n'`, shapes each
// line independently (gpui's `shape_line` rejects strings with
// `'\n'`), computes the caret's row + column from the byte offset,
// paints every line at its own Y, and renders one selection quad
// per row the selection spans. Caches the per-row layouts in
// `state.last_line_layouts` / `state.last_line_byte_ranges` /
// `state.last_line_height` so the IME's `bounds_for_range` and
// `character_index_for_point` work for multi-line.
// =====================================================================

/// The inner multi-line painter. Private to this module.
pub struct TextAreaElement {
    pub state: gpui::Entity<TextInputState>,
    pub focus_handle: FocusHandle,
    pub disabled: bool,
    pub text_color: Hsla,
    pub hint_color: Hsla,
    pub cursor_color: Hsla,
    pub selection_color: Hsla,
    pub placeholder: SharedString,
    /// Minimum visible height (the wrapper sets this so the
    /// text area is tappable even when empty). The actual height
    /// is `max(min_h, line_count * line_height)`.
    pub min_h: Pixels,
}

pub struct TextAreaPrepaintState {
    pub lines: Vec<ShapedLine>,
    /// Byte range in `value` for each row in `lines`. Length
    /// matches `lines`. The i-th range is
    /// `line_i_start..line_i_end`; non-final rows include the
    /// trailing `'\n'` in the range (so the next row's start
    /// is `range.end`).
    pub line_byte_ranges: Vec<Range<usize>>,
    pub selection_quads: Vec<PaintQuad>,
    pub cursor: Option<PaintQuad>,
    pub scroll_x: Pixels,
    pub cursor_visible: bool,
}

impl IntoElement for TextAreaElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextAreaElement {
    type RequestLayoutState = ();
    type PrepaintState = TextAreaPrepaintState;

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
        // Read the current value to know the line count. The
        // `use_keyed_state` machinery means the state is
        // persisted across re-renders, so reading it here is
        // cheap and always up-to-date for this frame.
        let value = self.state.read(cx).value.clone();
        let line_count = if value.is_empty() {
            1
        } else {
            value.split('\n').count()
        };
        let line_height_px = window.line_height();
        let total_height = line_height_px * line_count as f32;
        // Don't shrink below the wrapper's min_h. If the
        // content is shorter, the element is `min_h` tall with
        // the lines painted from the top; if longer, the
        // element grows to fit (and the wrapper's
        // `overflow_hidden()` clips any overflow beyond the
        // wrapper's own bounds).
        let height = total_height.max(self.min_h);

        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = height.into();
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
        let scroll_x_input = input.scroll_x;
        let cursor_visible = input.cursor_visible;

        let is_empty = value.is_empty();
        let placeholder = input.placeholder.clone();
        // Pick the display text. For `text_area` there's no
        // `value_override` (no masking), so it's always
        // `placeholder` when empty, `value` otherwise.
        let display_text: SharedString = if is_empty {
            placeholder
        } else {
            SharedString::from(value.clone())
        };

        // Split by `'\n'` and shape each line. `shape_line`
        // refuses strings containing `'\n'`, so the multi-line
        // case must break the value apart *first* and shape
        // each line individually.
        let line_strs: Vec<String> = if is_empty {
            vec![String::new()]
        } else {
            display_text.split('\n').map(String::from).collect()
        };

        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let run_color = if is_empty {
            self.hint_color
        } else {
            self.text_color
        };

        let mut shaped: Vec<ShapedLine> = Vec::with_capacity(line_strs.len());
        for line_str in &line_strs {
            let run = TextRun {
                len: line_str.len(),
                font: style.font(),
                color: run_color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let shaped_line =
                window
                    .text_system()
                    .shape_line(line_str.clone().into(), font_size, &[run], None);
            shaped.push(shaped_line);
        }

        // Build per-row byte ranges in `value`. For a value
        // "abc\ndef\n" the ranges are [0..3, 4..7, 8..8]:
        // row 0 is "abc" at 0..3, the next byte (3) is '\n' and
        // is included in row 0's range, row 1 starts at 4, etc.
        let line_byte_ranges: Vec<Range<usize>> = {
            let mut ranges = Vec::with_capacity(line_strs.len());
            let mut offset = 0usize;
            for (i, line_str) in line_strs.iter().enumerate() {
                let end = if i + 1 < line_strs.len() {
                    // Non-final row: include the '\n' in the
                    // range so the row covers
                    // `[line_start, line_start + line.len() + 1)`.
                    offset + line_str.len() + 1
                } else {
                    // Final row: end is just past the last char.
                    offset + line_str.len()
                };
                ranges.push(offset..end);
                offset = end;
            }
            ranges
        };

        // Find the caret's row + column. `caret` is a byte
        // offset in `value`. If the byte falls on a `'\n'`
        // (i.e. at the end of a non-final row), `position` with
        // `r.contains(&caret)` returns false for that row but
        // true for the next (the `'\n'` belongs to the
        // *previous* row's range, and the next row starts
        // right after it — so `caret == next_row.start` means
        // the caret is at the start of the next row). When
        // the caret is at `value.len()` past the last row,
        // `position` returns None and we fall back to the
        // last row.
        let caret_row = line_byte_ranges
            .iter()
            .position(|r| r.contains(&caret))
            .unwrap_or(line_byte_ranges.len().saturating_sub(1))
            .min(shaped.len().saturating_sub(1));
        let caret_col = caret.saturating_sub(line_byte_ranges[caret_row].start);

        let line_height_px = window.line_height();
        let caret_x_line = shaped[caret_row].x_for_index(caret_col);
        let cursor_thickness = px(2.0);
        let max_cursor_x = (bounds.size.width - cursor_thickness).max(Pixels::ZERO);
        // `max_scroll_x` is the horizontal scroll ceiling for
        // the *widest* line; smaller lines never need to
        // scroll past their own width.
        let max_line_width = shaped
            .iter()
            .map(|l| l.width)
            .fold(Pixels::ZERO, |a, b| a.max(b));
        let max_scroll_x = (max_line_width - max_cursor_x).max(Pixels::ZERO);
        let mut scroll_x = scroll_x_input.clamp(Pixels::ZERO, max_scroll_x);
        if caret_x_line < scroll_x {
            scroll_x = caret_x_line;
        } else if caret_x_line > scroll_x + max_cursor_x {
            scroll_x = caret_x_line - max_cursor_x;
        }
        scroll_x = scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        // Build selection quads — one per row the selection
        // touches. A selection that spans `value[1..10]` for a
        // value "abc\ndef\nghi" produces 3 quads (one per
        // row, clipped to the row's [start, end) range).
        let mut selection_quads = Vec::new();
        if !selection.is_empty() && !is_empty {
            for (i, line) in shaped.iter().enumerate() {
                let range = &line_byte_ranges[i];
                let sel_start = selection.start.max(range.start).min(range.end);
                let sel_end = selection.end.max(range.start).min(range.end);
                if sel_start < sel_end {
                    let col_start = sel_start - range.start;
                    let col_end = sel_end - range.start;
                    let start_x = line.x_for_index(col_start);
                    let end_x = line.x_for_index(col_end);
                    let y_top = bounds.top() + (i as f32) * line_height_px;
                    let y_bottom = y_top + line_height_px;
                    let quad = fill(
                        Bounds::from_corners(
                            point(bounds.left() + start_x.min(end_x) - scroll_x, y_top),
                            point(bounds.left() + start_x.max(end_x) - scroll_x, y_bottom),
                        ),
                        self.selection_color,
                    );
                    selection_quads.push(quad);
                }
            }
        }

        // The cursor quad is a single `Bounds` of
        // `cursor_thickness` width at the caret's (x, y) —
        // height is the line height of the row the caret
        // sits in.
        let cursor = if selection.is_empty() && !is_empty {
            let y_top = bounds.top() + (caret_row as f32) * line_height_px;
            let y_bottom = y_top + line_height_px;
            let cursor_paint_x = bounds.left() + caret_x_line - scroll_x;
            Some(fill(
                Bounds::new(
                    point(cursor_paint_x, y_top),
                    size(cursor_thickness, y_bottom - y_top),
                ),
                self.cursor_color,
            ))
        } else {
            None
        };

        TextAreaPrepaintState {
            lines: shaped,
            line_byte_ranges,
            selection_quads,
            cursor,
            scroll_x,
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
        // Register the IME / clipboard pipeline against the
        // element's bounds. The platform uses the
        // `EntityInputHandler` impl on `TextInputState` (which
        // sees the multi-line layouts via `last_line_*` and
        // computes row + column for both `bounds_for_range` and
        // `character_index_for_point`).
        if !self.disabled {
            window.handle_input(
                &self.focus_handle,
                ElementInputHandler::new(bounds, self.state.clone()),
                cx,
            );
        }

        // 1. Selection quads first (behind the text).
        let selection_quads = std::mem::take(&mut prepaint.selection_quads);
        for quad in selection_quads {
            window.paint_quad(quad);
        }

        // 2. Each line, shifted down by `row * line_height`.
        //    `scroll_x` is the horizontal offset (same value
        //    applies to every row — the area scrolls as a
        //    single block). Vertical scroll isn't implemented
        //    in v0.3 (the wrapper's `overflow_hidden()` clips
        //    any content past `min_h`).
        let line_height_px = window.line_height();
        let lines: Vec<ShapedLine> = std::mem::take(&mut prepaint.lines);
        let line_byte_ranges: Vec<Range<usize>> = std::mem::take(&mut prepaint.line_byte_ranges);
        for (i, line) in lines.iter().enumerate() {
            let y_offset = bounds.top() + (i as f32) * line_height_px;
            let origin_x = bounds.left() - prepaint.scroll_x;
            let _ = line.paint(point(origin_x, y_offset), line_height_px, window, cx);
        }

        // 3. The caret.
        let is_focused = self.focus_handle.is_focused(window);
        if is_focused
            && prepaint.cursor_visible
            && let Some(cur) = prepaint.cursor.take()
        {
            window.paint_quad(cur);
        }

        // 4. Cache the multi-line state for the IME.
        //    `last_line_layouts` and `last_line_byte_ranges`
        //    are the multi-line source of truth; `last_layout`
        //    is the first line, kept around for any code that
        //    still asks for the single-line slot.
        self.state.update(cx, |input, _cx| {
            input.last_layout = lines.first().cloned();
            input.last_line_layouts = lines;
            input.last_line_byte_ranges = line_byte_ranges;
            input.last_line_height = Some(line_height_px);
            input.last_bounds = Some(bounds);
            input.scroll_x = prepaint.scroll_x;
        });
    }
}

// =====================================================================
// `DefaultTextArea` — `headless::TextAreaProps` sugar.
// =====================================================================
